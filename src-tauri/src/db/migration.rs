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
    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_review_logs_card_id ON review_logs(card_id);
";
