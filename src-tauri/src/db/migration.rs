pub const DDL: &str = "
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

CREATE TABLE IF NOT EXISTS error_notebooks (
    id          TEXT PRIMARY KEY,
    remote_id   TEXT DEFAULT NULL,
    name        TEXT NOT NULL,
    sync_status TEXT NOT NULL DEFAULT 'local',
    version     INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at  TEXT DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS error_items (
    id                  TEXT PRIMARY KEY,
    remote_id           TEXT DEFAULT NULL,
    notebook_id         TEXT DEFAULT NULL,
    question_text       TEXT DEFAULT NULL,
    answer_text         TEXT DEFAULT NULL,
    analysis            TEXT DEFAULT NULL,
    wrong_answer_text   TEXT DEFAULT NULL,
    mistake_analysis    TEXT DEFAULT NULL,
    mistake_status      TEXT DEFAULT NULL,
    knowledge_points    TEXT NOT NULL DEFAULT '[]',
    user_notes          TEXT DEFAULT NULL,
    mastery_level       INTEGER NOT NULL DEFAULT 0,
    ef                  REAL NOT NULL DEFAULT 2.5,
    interval            INTEGER NOT NULL DEFAULT 1,
    repetitions         INTEGER NOT NULL DEFAULT 0,
    next_review         TEXT NOT NULL DEFAULT (date('now')),
    sync_status         TEXT NOT NULL DEFAULT 'local_draft',
    version             INTEGER NOT NULL DEFAULT 0,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at          TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at          TEXT DEFAULT NULL,
    FOREIGN KEY (notebook_id) REFERENCES error_notebooks(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS error_item_images (
    id              TEXT PRIMARY KEY,
    error_item_id   TEXT NOT NULL,
    local_path      TEXT NOT NULL,
    remote_key      TEXT DEFAULT NULL,
    remote_url      TEXT DEFAULT NULL,
    sha256          TEXT NOT NULL,
    mime_type       TEXT NOT NULL DEFAULT 'image/jpeg',
    upload_status   TEXT NOT NULL DEFAULT 'local',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (error_item_id) REFERENCES error_items(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS error_review_logs (
    id                  TEXT PRIMARY KEY,
    remote_id           TEXT DEFAULT NULL,
    error_item_id        TEXT NOT NULL,
    quality             INTEGER NOT NULL,
    reviewed_at         TEXT NOT NULL DEFAULT (datetime('now')),
    duration_seconds    INTEGER NOT NULL DEFAULT 0,
    ef_before           REAL NOT NULL,
    ef_after            REAL NOT NULL,
    mastery_level       INTEGER NOT NULL,
    next_review         TEXT NOT NULL,
    sync_status         TEXT NOT NULL DEFAULT 'pending_sync',
    FOREIGN KEY (error_item_id) REFERENCES error_items(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS sync_state (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_error_items_next_review ON error_items(next_review);
CREATE INDEX IF NOT EXISTS idx_error_items_sync_status ON error_items(sync_status);
CREATE INDEX IF NOT EXISTS idx_error_items_remote_id ON error_items(remote_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_error_item_images_item_id ON error_item_images(error_item_id);
CREATE INDEX IF NOT EXISTS idx_error_review_logs_sync_status ON error_review_logs(sync_status);
";

pub fn run(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(DDL)?;

    let has_duration = conn
        .prepare("PRAGMA table_info(review_logs)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(Result::ok)
        .any(|name| name == "duration_seconds");

    if !has_duration {
        conn.execute(
            "ALTER TABLE review_logs ADD COLUMN duration_seconds INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    Ok(())
}
