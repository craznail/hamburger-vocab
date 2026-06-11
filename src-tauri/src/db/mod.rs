use chrono::Datelike;
use rusqlite::Connection;
use std::sync::Mutex;
use uuid::Uuid;

pub mod card_repo;
pub mod deck_repo;
pub mod migration;
pub mod models;

pub struct DbState {
    pub conn: Mutex<Connection>,
}

// ---- Helpers ----

pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn today_str() -> String {
    let now = chrono::Local::now();
    format!("{}-{:02}-{:02}", now.year(), now.month() as u8, now.day())
}

pub fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

// ---- Init ----

pub fn init_db(db_path: &std::path::Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON")?;
    conn.execute_batch(migration::DDL)?;
    Ok(conn)
}

