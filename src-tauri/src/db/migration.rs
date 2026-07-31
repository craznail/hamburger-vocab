use rusqlite::{params, Connection, OptionalExtension};

const ERROR_SYNC_SCHEMA_VERSION: &str = "4";

const CORE_DDL: &str = "
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
    duration_seconds INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_review_logs_card_id ON review_logs(card_id);
CREATE INDEX IF NOT EXISTS idx_review_logs_reviewed_at ON review_logs(reviewed_at);

CREATE TABLE IF NOT EXISTS practice_logs (
    id               TEXT PRIMARY KEY,
    card_id          TEXT NOT NULL,
    practiced_at     TEXT NOT NULL DEFAULT (datetime('now')),
    quality          INTEGER NOT NULL,
    duration_seconds INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_practice_logs_card_id ON practice_logs(card_id);
CREATE INDEX IF NOT EXISTS idx_practice_logs_practiced_at ON practice_logs(practiced_at);

CREATE TABLE IF NOT EXISTS sync_state (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);
";

const ERROR_SYNC_DDL: &str = "
CREATE TABLE IF NOT EXISTS error_notebooks (
    id          TEXT PRIMARY KEY,
    remote_id   TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS error_items (
    id                    TEXT PRIMARY KEY,
    remote_id             TEXT DEFAULT NULL UNIQUE,
    notebook_id           TEXT DEFAULT NULL,
    question_text         TEXT DEFAULT NULL,
    answer_text           TEXT DEFAULT NULL,
    analysis              TEXT DEFAULT NULL,
    wrong_answer_text     TEXT DEFAULT NULL,
    mistake_analysis      TEXT DEFAULT NULL,
    mistake_status        TEXT DEFAULT NULL,
    knowledge_points      TEXT NOT NULL DEFAULT '[]',
    user_notes            TEXT DEFAULT NULL,
    mastery_level         INTEGER NOT NULL DEFAULT 0,
    ef                    REAL NOT NULL DEFAULT 2.5,
    interval              INTEGER NOT NULL DEFAULT 1,
    repetitions           INTEGER NOT NULL DEFAULT 0,
    next_review           TEXT NOT NULL DEFAULT (date('now')),
    analysis_status       TEXT NOT NULL DEFAULT 'pending_analysis',
    remote_version        INTEGER NOT NULL DEFAULT 0,
    server_snapshot_json  TEXT DEFAULT NULL,
    created_at            TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at            TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at            TEXT DEFAULT NULL,
    FOREIGN KEY (notebook_id) REFERENCES error_notebooks(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS error_item_images (
    id              TEXT PRIMARY KEY,
    error_item_id   TEXT NOT NULL UNIQUE,
    local_path      TEXT DEFAULT NULL,
    remote_key      TEXT DEFAULT NULL,
    remote_url      TEXT DEFAULT NULL,
    sha256          TEXT NOT NULL,
    mime_type       TEXT NOT NULL DEFAULT 'image/jpeg',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (error_item_id) REFERENCES error_items(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS error_review_logs (
    id                  TEXT PRIMARY KEY,
    error_item_id       TEXT NOT NULL,
    quality             INTEGER NOT NULL,
    reviewed_at         TEXT NOT NULL DEFAULT (datetime('now')),
    duration_seconds    INTEGER NOT NULL DEFAULT 0,
    ef_before           REAL NOT NULL,
    ef_after            REAL NOT NULL,
    mastery_level       INTEGER NOT NULL,
    next_review         TEXT NOT NULL,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (error_item_id) REFERENCES error_items(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS error_sync_ops (
    op_id               TEXT PRIMARY KEY,
    entity_type         TEXT NOT NULL,
    action              TEXT NOT NULL,
    local_item_id       TEXT NOT NULL,
    remote_item_id      TEXT DEFAULT NULL,
    base_version        INTEGER DEFAULT NULL,
    payload_json        TEXT NOT NULL,
    client_timestamp    TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'pending',
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_error_sync_ops_status ON error_sync_ops(status, client_timestamp);
CREATE INDEX IF NOT EXISTS idx_error_sync_ops_local_item ON error_sync_ops(local_item_id, status);

CREATE TABLE IF NOT EXISTS error_sync_conflicts (
    id                    TEXT PRIMARY KEY,
    local_item_id         TEXT NOT NULL UNIQUE,
    op_id                 TEXT NOT NULL UNIQUE,
    server_version        INTEGER NOT NULL,
    server_snapshot_json  TEXT NOT NULL,
    error_code            TEXT DEFAULT NULL,
    created_at            TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at            TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (local_item_id) REFERENCES error_items(id) ON DELETE CASCADE,
    FOREIGN KEY (op_id) REFERENCES error_sync_ops(op_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_error_items_remote_id ON error_items(remote_id);
CREATE INDEX IF NOT EXISTS idx_error_items_next_review ON error_items(next_review);
CREATE INDEX IF NOT EXISTS idx_error_items_deleted_at ON error_items(deleted_at);
CREATE INDEX IF NOT EXISTS idx_error_review_logs_item_id ON error_review_logs(error_item_id);
";

fn current_error_sync_schema_version(conn: &Connection) -> Result<Option<String>, rusqlite::Error> {
    conn.query_row(
        "SELECT value FROM sync_state WHERE key = 'error_sync_schema_version'",
        [],
        |row| row.get(0),
    )
    .optional()
}

fn set_error_sync_schema_version(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO sync_state (key, value) VALUES ('error_sync_schema_version', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![ERROR_SYNC_SCHEMA_VERSION],
    )?;
    Ok(())
}

fn table_has_column(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
) -> Result<bool, rusqlite::Error> {
    let pragma = format!("PRAGMA table_info({table_name})");
    let mut stmt = conn.prepare(&pragma)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        if row? == column_name {
            return Ok(true);
        }
    }
    Ok(false)
}

fn cleanup_local_only_sync_state(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM error_sync_conflicts
         WHERE local_item_id IN (
           SELECT id
           FROM error_items
           WHERE remote_id IS NULL
             AND analysis_status IN ('pending_analysis', 'analyze_failed')
         )",
        [],
    )?;
    conn.execute(
        "DELETE FROM error_sync_ops
         WHERE action IN ('create', 'update')
           AND local_item_id IN (
             SELECT id
             FROM error_items
             WHERE remote_id IS NULL
               AND analysis_status IN ('pending_analysis', 'analyze_failed')
           )",
        [],
    )?;
    Ok(())
}

fn reset_error_sync_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        DROP TABLE IF EXISTS error_sync_conflicts;
        DROP TABLE IF EXISTS error_sync_ops;
        DROP TABLE IF EXISTS error_review_logs;
        DROP TABLE IF EXISTS error_item_images;
        DROP TABLE IF EXISTS error_items;
        DROP TABLE IF EXISTS error_notebooks;
        ",
    )?;
    conn.execute_batch(ERROR_SYNC_DDL)?;
    conn.execute(
        "DELETE FROM sync_state WHERE key IN ('last_error_sync_at', 'last_error_sync_cursor')",
        [],
    )?;
    set_error_sync_schema_version(conn)?;
    Ok(())
}

fn migrate_error_sync_schema_from_v3(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(ERROR_SYNC_DDL)?;
    if !table_has_column(conn, "error_sync_conflicts", "error_code")? {
        conn.execute(
            "ALTER TABLE error_sync_conflicts ADD COLUMN error_code TEXT DEFAULT NULL",
            [],
        )?;
    }
    cleanup_local_only_sync_state(conn)?;
    set_error_sync_schema_version(conn)?;
    Ok(())
}

pub fn run(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(CORE_DDL)?;

    match current_error_sync_schema_version(conn)?.as_deref() {
        Some("4") => {
            conn.execute_batch(ERROR_SYNC_DDL)?;
            if !table_has_column(conn, "error_sync_conflicts", "error_code")? {
                conn.execute(
                    "ALTER TABLE error_sync_conflicts ADD COLUMN error_code TEXT DEFAULT NULL",
                    [],
                )?;
            }
            cleanup_local_only_sync_state(conn)?;
        }
        Some("3") => migrate_error_sync_schema_from_v3(conn)?,
        _ => reset_error_sync_schema(conn)?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resets_legacy_error_sync_schema_to_new_version() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(CORE_DDL).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE error_items (id TEXT PRIMARY KEY, sync_status TEXT);
            INSERT INTO sync_state (key, value) VALUES ('error_sync_schema_version', '1');
            ",
        )
        .unwrap();

        run(&conn).unwrap();

        let version = current_error_sync_schema_version(&conn).unwrap();
        assert_eq!(version.as_deref(), Some(ERROR_SYNC_SCHEMA_VERSION));

        let has_analysis_status = conn
            .prepare("PRAGMA table_info(error_items)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .filter_map(Result::ok)
            .any(|name| name == "analysis_status");
        assert!(has_analysis_status);
    }

    #[test]
    fn migrates_v3_and_cleans_local_only_invalid_sync_state() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(CORE_DDL).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE error_notebooks (
                id TEXT PRIMARY KEY,
                remote_id TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE error_items (
                id TEXT PRIMARY KEY,
                remote_id TEXT DEFAULT NULL UNIQUE,
                notebook_id TEXT DEFAULT NULL,
                question_text TEXT DEFAULT NULL,
                answer_text TEXT DEFAULT NULL,
                analysis TEXT DEFAULT NULL,
                wrong_answer_text TEXT DEFAULT NULL,
                mistake_analysis TEXT DEFAULT NULL,
                mistake_status TEXT DEFAULT NULL,
                knowledge_points TEXT NOT NULL DEFAULT '[]',
                user_notes TEXT DEFAULT NULL,
                mastery_level INTEGER NOT NULL DEFAULT 0,
                ef REAL NOT NULL DEFAULT 2.5,
                interval INTEGER NOT NULL DEFAULT 1,
                repetitions INTEGER NOT NULL DEFAULT 0,
                next_review TEXT NOT NULL DEFAULT (date('now')),
                analysis_status TEXT NOT NULL DEFAULT 'pending_analysis',
                remote_version INTEGER NOT NULL DEFAULT 0,
                server_snapshot_json TEXT DEFAULT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                deleted_at TEXT DEFAULT NULL
            );
            CREATE TABLE error_sync_ops (
                op_id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                action TEXT NOT NULL,
                local_item_id TEXT NOT NULL,
                remote_item_id TEXT DEFAULT NULL,
                base_version INTEGER DEFAULT NULL,
                payload_json TEXT NOT NULL,
                client_timestamp TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE error_sync_conflicts (
                id TEXT PRIMARY KEY,
                local_item_id TEXT NOT NULL UNIQUE,
                op_id TEXT NOT NULL UNIQUE,
                server_version INTEGER NOT NULL,
                server_snapshot_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            INSERT INTO error_items (id, remote_id, analysis_status, created_at, updated_at)
            VALUES
              ('local-failed', NULL, 'analyze_failed', '2026-06-27 20:00:00', '2026-06-27 20:00:00'),
              ('remote-ready', 'remote-1', 'ready', '2026-06-27 20:00:00', '2026-06-27 20:00:00');
            INSERT INTO error_sync_ops (op_id, entity_type, action, local_item_id, payload_json, client_timestamp, status, created_at, updated_at)
            VALUES
              ('op-local', 'error_item', 'create', 'local-failed', '{}', '2026-06-27 20:00:00', 'conflicted', '2026-06-27 20:00:00', '2026-06-27 20:00:00'),
              ('op-remote', 'error_item', 'update', 'remote-ready', '{}', '2026-06-27 20:00:00', 'pending', '2026-06-27 20:00:00', '2026-06-27 20:00:00');
            INSERT INTO error_sync_conflicts (id, local_item_id, op_id, server_version, server_snapshot_json, created_at, updated_at)
            VALUES
              ('conflict-local', 'local-failed', 'op-local', 0, 'null', '2026-06-27 20:00:00', '2026-06-27 20:00:00');
            INSERT INTO sync_state (key, value) VALUES ('error_sync_schema_version', '3');
            ",
        )
        .unwrap();

        run(&conn).unwrap();

        let version = current_error_sync_schema_version(&conn).unwrap();
        assert_eq!(version.as_deref(), Some(ERROR_SYNC_SCHEMA_VERSION));

        let has_error_code = table_has_column(&conn, "error_sync_conflicts", "error_code").unwrap();
        assert!(has_error_code);

        let local_ops: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM error_sync_ops WHERE local_item_id = 'local-failed'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let local_conflicts: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM error_sync_conflicts WHERE local_item_id = 'local-failed'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let remote_ops: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM error_sync_ops WHERE local_item_id = 'remote-ready'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(local_ops, 0);
        assert_eq!(local_conflicts, 0);
        assert_eq!(remote_ops, 1);
    }
}
