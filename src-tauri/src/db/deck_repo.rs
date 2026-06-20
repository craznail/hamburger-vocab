use rusqlite::{params, Connection};

use super::models::*;
use super::{generate_id, today_str};

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
            SUM(CASE WHEN c.next_review <= ?1 THEN 1 ELSE 0 END) AS due_count
         FROM decks d
         LEFT JOIN cards c ON c.deck_id = d.id
         GROUP BY d.id
         ORDER BY d.created_at DESC",
    )?;
    let today = today_str();
    let rows = stmt.query_map(params![today], |row| {
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
    // ON DELETE CASCADE handles cards and review_logs automatically
    conn.execute("DELETE FROM decks WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_deck_by_id(
    conn: &Connection,
    deck_id: &str,
) -> Result<Option<DeckInfo>, rusqlite::Error> {
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
            SUM(CASE WHEN next_review <= ?2 THEN 1 ELSE 0 END) AS due
         FROM cards WHERE deck_id = ?1",
    )?;
    let stats = stmt.query_row(params![deck_id, today_str()], |row| {
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
