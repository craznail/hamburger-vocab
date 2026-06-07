use chrono::Datelike;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

const DDL: &str = "
CREATE TABLE IF NOT EXISTS decks (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS cards (
    id            TEXT PRIMARY KEY,
    deck_id       TEXT NOT NULL,
    word          TEXT NOT NULL,
    inflections   TEXT DEFAULT '',
    definition    TEXT DEFAULT '',
    ef            REAL NOT NULL DEFAULT 2.5,
    interval      INTEGER NOT NULL DEFAULT 1,
    repetitions   INTEGER NOT NULL DEFAULT 0,
    next_review   TEXT NOT NULL DEFAULT (date('now')),
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    last_review_at TEXT DEFAULT NULL,
    FOREIGN KEY (deck_id) REFERENCES decks(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_cards_deck_word ON cards(deck_id, word);
CREATE INDEX IF NOT EXISTS idx_cards_next_review ON cards(next_review);
CREATE INDEX IF NOT EXISTS idx_cards_deck_id ON cards(deck_id);

CREATE TABLE IF NOT EXISTS review_logs (
    id            TEXT PRIMARY KEY,
    card_id       TEXT NOT NULL,
    reviewed_at   TEXT NOT NULL DEFAULT (datetime('now')),
    quality       INTEGER NOT NULL,
    ef_before     REAL NOT NULL,
    ef_after      REAL NOT NULL,
    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_review_logs_card_id ON review_logs(card_id);
";

pub struct DbState {
    pub conn: Mutex<Connection>,
}

// ---- Data types ----

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Deck {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub word_count: i64,
    pub mastered_count: i64,
    pub due_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeckInfo {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    pub deck_id: String,
    pub word: String,
    pub inflections: String,
    pub definition: String,
    pub ef: f64,
    pub interval: i64,
    pub repetitions: i64,
    pub next_review: String,
    pub created_at: String,
    pub last_review_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TodayCard {
    pub id: String,
    pub word: String,
    pub inflections: String,
    pub definition: String,
    pub ef: f64,
    pub interval: i64,
    pub repetitions: i64,
    pub next_review: String,
    pub created_at: String,
    pub last_review_at: Option<String>,
    pub deck_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeckStats {
    pub total: i64,
    pub mastered: i64,
    pub due: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardImport {
    pub word: String,
    pub inflections: Vec<String>,
    pub definition: String,
}

// ---- Helpers ----

fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

fn today_str() -> String {
    let now = chrono::Local::now();
    format!("{}-{:02}-{:02}", now.year(), now.month() as u8, now.day())
}

fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

// ---- Init ----

pub fn init_db(db_path: &std::path::Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON")?;
    conn.execute_batch(DDL)?;
    Ok(conn)
}

// ---- Decks ----

pub fn create_deck(conn: &Connection, name: &str) -> Result<String, rusqlite::Error> {
    let id = generate_id();
    conn.execute(
        "INSERT INTO decks (id, name) VALUES (?1, ?2)",
        params![id, name],
    )?;
    Ok(id)
}

pub fn get_decks(conn: &Connection) -> Result<Vec<Deck>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT d.id, d.name, d.created_at,
            COUNT(c.id) AS word_count,
            SUM(CASE WHEN c.repetitions >= 2 THEN 1 ELSE 0 END) AS mastered_count,
            SUM(CASE WHEN c.next_review <= date('now') AND c.repetitions < 2 THEN 1 ELSE 0 END) AS due_count
         FROM decks d
         LEFT JOIN cards c ON c.deck_id = d.id
         GROUP BY d.id
         ORDER BY d.created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Deck {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
            word_count: row.get::<_, Option<i64>>(3)?.unwrap_or(0),
            mastered_count: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
            due_count: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn delete_deck(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM cards WHERE deck_id = ?1", params![id])?;
    conn.execute("DELETE FROM decks WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_deck_by_id(conn: &Connection, deck_id: &str) -> Result<Option<DeckInfo>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, name, created_at FROM decks WHERE id = ?1")?;
    let mut rows = stmt.query_map(params![deck_id], |row| {
        Ok(DeckInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;
    match rows.next() {
        Some(Ok(d)) => Ok(Some(d)),
        _ => Ok(None),
    }
}

pub fn get_deck_stats(conn: &Connection, deck_id: &str) -> Result<DeckStats, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT
            COUNT(*) AS total,
            SUM(CASE WHEN repetitions >= 2 THEN 1 ELSE 0 END) AS mastered,
            SUM(CASE WHEN next_review <= date('now') AND repetitions < 2 THEN 1 ELSE 0 END) AS due
         FROM cards WHERE deck_id = ?1",
    )?;
    let stats = stmt.query_row(params![deck_id], |row| {
        Ok(DeckStats {
            total: row.get::<_, Option<i64>>(0)?.unwrap_or(0),
            mastered: row.get::<_, Option<i64>>(1)?.unwrap_or(0),
            due: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
        })
    })?;
    Ok(stats)
}

pub fn get_today_count(conn: &Connection) -> Result<i64, rusqlite::Error> {
    let today = today_str();
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM cards WHERE next_review <= ?1")?;
    let count = stmt.query_row(params![today], |row| row.get::<_, i64>(0))?;
    Ok(count)
}

// ---- Cards ----

pub fn import_cards(conn: &Connection, deck_id: &str, cards: &[CardImport]) -> Result<(), rusqlite::Error> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO cards (id, deck_id, word, inflections, definition)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for card in cards {
        let inflections_json = serde_json::to_string(&card.inflections).unwrap_or_default();
        stmt.execute(params![
            generate_id(),
            deck_id,
            &card.word,
            inflections_json,
            &card.definition,
        ])?;
    }
    Ok(())
}

pub fn get_cards_by_deck_id(conn: &Connection, deck_id: &str) -> Result<Vec<Card>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, deck_id, word, inflections, definition, ef, interval, repetitions,
                next_review, created_at, last_review_at
         FROM cards WHERE deck_id = ?1
         ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![deck_id], |row| {
        Ok(Card {
            id: row.get(0)?,
            deck_id: row.get(1)?,
            word: row.get(2)?,
            inflections: row.get(3)?,
            definition: row.get(4)?,
            ef: row.get(5)?,
            interval: row.get(6)?,
            repetitions: row.get(7)?,
            next_review: row.get(8)?,
            created_at: row.get(9)?,
            last_review_at: row.get(10)?,
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

pub fn get_today_cards(conn: &Connection, deck_id: Option<&str>) -> Result<Vec<TodayCard>, rusqlite::Error> {
    let today = today_str();
    let (sql, param_values): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(did) = deck_id {
        (
            "SELECT c.id, c.word, c.inflections, c.definition, c.ef, c.interval, c.repetitions,
                    c.next_review, c.created_at, c.last_review_at, d.name AS deck_name
             FROM cards c
             JOIN decks d ON d.id = c.deck_id
             WHERE c.deck_id = ?1 AND c.next_review <= ?2
             ORDER BY c.ef ASC, c.next_review ASC LIMIT 50"
                .to_string(),
            vec![Box::new(did.to_string()), Box::new(today)],
        )
    } else {
        (
            "SELECT c.id, c.word, c.inflections, c.definition, c.ef, c.interval, c.repetitions,
                    c.next_review, c.created_at, c.last_review_at, d.name AS deck_name
             FROM cards c
             JOIN decks d ON d.id = c.deck_id
             WHERE c.next_review <= ?1
             ORDER BY c.ef ASC, c.next_review ASC LIMIT 50"
                .to_string(),
            vec![Box::new(today)],
        )
    };

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(TodayCard {
            id: row.get(0)?,
            word: row.get(1)?,
            inflections: row.get(2)?,
            definition: row.get(3)?,
            ef: row.get(4)?,
            interval: row.get(5)?,
            repetitions: row.get(6)?,
            next_review: row.get(7)?,
            created_at: row.get(8)?,
            last_review_at: row.get(9)?,
            deck_name: row.get(10)?,
        })
    })?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row?);
    }
    Ok(result)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewUpdate {
    pub ef: f64,
    pub interval: i64,
    pub repetitions: i64,
    pub next_review: String,
}

pub fn update_card_after_review(
    conn: &Connection,
    card_id: &str,
    update: &ReviewUpdate,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE cards SET ef = ?1, interval = ?2, repetitions = ?3, next_review = ?4, last_review_at = ?5
         WHERE id = ?6",
        params![
            update.ef,
            update.interval,
            update.repetitions,
            update.next_review,
            now_str(),
            card_id,
        ],
    )?;
    Ok(())
}

// ---- Review Logs ----

pub fn add_review_log(
    conn: &Connection,
    card_id: &str,
    quality: i64,
    ef_before: f64,
    ef_after: f64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO review_logs (id, card_id, quality, ef_before, ef_after) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![generate_id(), card_id, quality, ef_before, ef_after],
    )?;
    Ok(())
}
