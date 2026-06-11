use rusqlite::{params, Connection};

use super::models::*;
use super::{generate_id, now_str, today_str};

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

pub fn get_card_by_id(conn: &Connection, card_id: &str) -> Result<Option<Card>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, deck_id, word, inflections, definition, ef, interval, repetitions,
                next_review, created_at, last_review_at
         FROM cards WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![card_id], |row| {
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
    match rows.next() {
        Some(Ok(c)) => Ok(Some(c)),
        _ => Ok(None),
    }
}

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
