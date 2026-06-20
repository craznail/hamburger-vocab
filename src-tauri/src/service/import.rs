use crate::db::card_repo;
use crate::db::deck_repo;
use crate::db::models::CardImport;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ParsedRow {
    pub word: String,
    pub inflections: Vec<String>,
    pub definition: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ParseError {
    pub line: usize,
    pub text: String,
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ParseResult {
    pub format: String,
    pub rows: Vec<ParsedRow>,
    pub preview: Vec<ParsedRow>,
    pub errors: Vec<ParseError>,
    pub total_lines: usize,
    pub valid_count: usize,
}

/// Parse text content as block format separated by blank lines:
///    line 1 = word
///    middle lines = inflections
///    last line = definition
pub fn parse_txt_content(text: &str) -> ParseResult {
    let mut rows = Vec::new();
    let mut errors = Vec::new();
    let mut current_block: Vec<(usize, String)> = Vec::new();
    let mut total_lines = 0usize;

    for (index, raw_line) in text.lines().enumerate() {
        let line_num = index + 1;
        let trimmed = raw_line.trim();

        if trimmed.is_empty() {
            if !current_block.is_empty() {
                push_block_row(&current_block, &mut rows, &mut errors);
                current_block.clear();
            }
            continue;
        }

        total_lines += 1;
        current_block.push((line_num, trimmed.to_string()));
    }

    if !current_block.is_empty() {
        push_block_row(&current_block, &mut rows, &mut errors);
    }

    let preview: Vec<ParsedRow> = rows.iter().take(3).cloned().collect();
    let valid_count = rows.len();

    ParseResult {
        format: "D".to_string(),
        rows,
        preview,
        errors,
        total_lines,
        valid_count,
    }
}

fn push_block_row(
    block: &[(usize, String)],
    rows: &mut Vec<ParsedRow>,
    errors: &mut Vec<ParseError>,
) {
    if block.is_empty() {
        return;
    }

    let word = block[0].1.trim().to_string();
    if word.is_empty() {
        errors.push(ParseError {
            line: block[0].0,
            text: String::new(),
            msg: "单词不能为空".to_string(),
        });
        return;
    }

    match block.len() {
        1 => rows.push(ParsedRow {
            word,
            inflections: vec![],
            definition: String::new(),
        }),
        2 => rows.push(ParsedRow {
            word,
            inflections: vec![],
            definition: block[1].1.trim().to_string(),
        }),
        _ => {
            let definition = block
                .last()
                .map(|(_, line)| line.trim().to_string())
                .unwrap_or_default();

            if definition.is_empty() {
                errors.push(ParseError {
                    line: block.last().map(|(line, _)| *line).unwrap_or(block[0].0),
                    text: block
                        .last()
                        .map(|(_, line)| line.clone())
                        .unwrap_or_default(),
                    msg: "缺少释义".to_string(),
                });
                return;
            }

            let inflections = block[1..block.len() - 1]
                .iter()
                .map(|(_, line)| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();

            rows.push(ParsedRow {
                word,
                inflections,
                definition,
            });
        }
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

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ImportFromTextResult {
    pub deck_id: String,
    pub deck_name: String,
    pub count: usize,
    pub parse_result: ParseResult,
}

#[cfg(test)]
mod tests {
    use super::parse_txt_content;

    #[test]
    fn parses_block_format_with_blank_lines() {
        let text = "am\nwas been\n是\n\nis\nwas been\n是";
        let result = parse_txt_content(text);

        assert_eq!(result.format, "D");
        assert_eq!(result.valid_count, 2);
        assert_eq!(result.rows[0].word, "am");
        assert_eq!(result.rows[0].inflections, vec!["was been"]);
        assert_eq!(result.rows[0].definition, "是");
        assert_eq!(result.rows[1].word, "is");
    }
}
