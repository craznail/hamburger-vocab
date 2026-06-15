use crate::db::card_repo;
use crate::db::deck_repo;
use crate::db::models::CardImport;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedRow {
    pub word: String,
    pub inflections: Vec<String>,
    pub definition: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParseError {
    pub line: usize,
    pub text: String,
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParseResult {
    pub format: String,
    pub rows: Vec<ParsedRow>,
    pub preview: Vec<ParsedRow>,
    pub errors: Vec<ParseError>,
    pub total_lines: usize,
    pub valid_count: usize,
}

/// Parse text content and detect format (A: word only, B: word+definition, C: word+inflections+definition)
pub fn parse_txt_content(text: &str) -> ParseResult {
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if lines.is_empty() {
        return ParseResult {
            format: "unknown".to_string(),
            rows: vec![],
            preview: vec![],
            errors: vec![],
            total_lines: 0,
            valid_count: 0,
        };
    }

    // Count fields per line to detect format
    let field_counts: Vec<usize> = lines.iter().map(|l| l.split_whitespace().count()).collect();

    let count1 = field_counts.iter().filter(|&&c| c == 1).count();
    let count2 = field_counts.iter().filter(|&&c| c == 2).count();
    let count3plus = field_counts.iter().filter(|&&c| c >= 3).count();

    let format = if count1 >= count2 && count1 >= count3plus {
        "A".to_string()
    } else if count2 >= count1 && count2 >= count3plus {
        "B".to_string()
    } else {
        "C".to_string()
    };

    let mut rows = Vec::new();
    let mut errors = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let line_num = i + 1;

        if format == "A" {
            rows.push(ParsedRow {
                word: parts[0].to_string(),
                inflections: vec![],
                definition: String::new(),
            });
        } else if format == "B" {
            if parts.len() < 2 {
                errors.push(ParseError {
                    line: line_num,
                    text: line.to_string(),
                    msg: "缺少释义".to_string(),
                });
                rows.push(ParsedRow {
                    word: parts[0].to_string(),
                    inflections: vec![],
                    definition: String::new(),
                });
            } else {
                rows.push(ParsedRow {
                    word: parts[0].to_string(),
                    inflections: vec![],
                    definition: parts[1..].join(" "),
                });
            }
        } else if format == "C" {
            if parts.len() < 3 {
                errors.push(ParseError {
                    line: line_num,
                    text: line.to_string(),
                    msg: "格式 C 需要至少 3 个字段：单词 词形变化 释义".to_string(),
                });
                continue;
            }
            rows.push(ParsedRow {
                word: parts[0].to_string(),
                inflections: parts[1..parts.len() - 1].iter().map(|s| s.to_string()).collect(),
                definition: parts[parts.len() - 1].to_string(),
            });
        }
    }

    // Check for mixed format
    for (i, line) in lines.iter().enumerate() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if format == "A" && parts.len() != 1 {
            errors.push(ParseError {
                line: i + 1,
                text: line.to_string(),
                msg: format!("该行有 {} 个字段，但文件格式为 A（纯单词）", parts.len()),
            });
        } else if format == "B" && parts.len() < 2 {
            errors.push(ParseError {
                line: i + 1,
                text: line.to_string(),
                msg: "该行只有 1 个字段，但文件格式为 B（单词+释义）".to_string(),
            });
        }
    }

    let preview: Vec<ParsedRow> = rows.iter().take(3).cloned().collect();
    let total_lines = lines.len();
    let valid_count = rows.len();

    ParseResult {
        format,
        rows,
        preview,
        errors,
        total_lines,
        valid_count,
    }
}

/// Import from raw text: parse + create deck + import cards, all in one operation.
/// Executes in a transaction to ensure atomicity.
pub fn import_from_text(
    conn: &rusqlite::Connection,
    deck_name: &str,
    text: &str,
) -> AppResult<ImportFromTextResult> {
    let parse_result = parse_txt_content(text);

    if parse_result.rows.is_empty() {
        return Err(AppError::InvalidInput("文件中没有有效的单词".into()));
    }

    // Execute create deck + import cards in a transaction
    conn.execute_batch("BEGIN IMMEDIATE")?;

    let result = (|| -> Result<ImportFromTextResult, rusqlite::Error> {
        let deck_id = deck_repo::create_deck(conn, deck_name)?;

        let cards: Vec<CardImport> = parse_result
            .rows
            .iter()
            .map(|r| CardImport {
                word: r.word.clone(),
                inflections: r.inflections.clone(),
                definition: r.definition.clone(),
            })
            .collect();

        let count = cards.len();
        card_repo::import_cards(conn, &deck_id, &cards)?;

        Ok(ImportFromTextResult {
            deck_id,
            deck_name: deck_name.to_string(),
            count,
            parse_result,
        })
    })();

    match result {
        Ok(r) => {
            conn.execute_batch("COMMIT")?;
            Ok(r)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(AppError::Database(e))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFromTextResult {
    pub deck_id: String,
    pub deck_name: String,
    pub count: usize,
    pub parse_result: ParseResult,
}
