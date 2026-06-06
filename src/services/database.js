let db = null

function generateId() {
  return crypto.randomUUID()
}

function todayStr() {
  const d = new Date()
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

function nowStr() {
  return new Date().toISOString().replace('T', ' ').slice(0, 19)
}

const STORAGE_KEY = 'vocab_db_snapshot'

function loadFromStorage() {
  try {
    const data = localStorage.getItem(STORAGE_KEY)
    if (data) {
      const bytes = Uint8Array.from(atob(data), c => c.charCodeAt(0))
      return bytes
    }
  } catch (e) {
    console.warn('Failed to load DB from storage', e)
  }
  return null
}

function saveToStorage(dbInstance) {
  try {
    const data = dbInstance.export()
    const binary = Array.from(data).map(b => String.fromCharCode(b)).join('')
    localStorage.setItem(STORAGE_KEY, btoa(binary))
  } catch (e) {
    console.error('Failed to save DB to storage', e)
  }
}

const DDL = `
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
`

export async function initDB() {
  // Dynamically import sql.js so Vite can properly handle CJS conversion
  const sqlJsModule = await import('sql.js')
  const initSqlJs = sqlJsModule.default || sqlJsModule
  const SQL = await initSqlJs({ locateFile: () => '/sql-wasm-browser.wasm' })

  const saved = loadFromStorage()
  if (saved) {
    db = new SQL.Database(saved)
  } else {
    db = new SQL.Database()
  }

  db.run('PRAGMA foreign_keys = ON')
  db.run(DDL)

  if (!saved) {
    saveToStorage(db)
  }
}

function ensureDB() {
  if (!db) throw new Error('Database not initialized. Call initDB() first.')
  return db
}

// ---- Decks ----

export function createDeck(name) {
  const d = ensureDB()
  const id = generateId()
  d.run('INSERT INTO decks (id, name) VALUES (?, ?)', [id, name])
  saveToStorage(d)
  return id
}

export function getDecks() {
  const d = ensureDB()
  const rows = d.exec(`
    SELECT d.id, d.name, d.created_at,
      COUNT(c.id) AS word_count,
      SUM(CASE WHEN c.repetitions >= 2 THEN 1 ELSE 0 END) AS mastered_count,
      SUM(CASE WHEN c.next_review <= date('now') AND c.repetitions < 2 THEN 1 ELSE 0 END) AS due_count
    FROM decks d
    LEFT JOIN cards c ON c.deck_id = d.id
    GROUP BY d.id
    ORDER BY d.created_at DESC
  `)
  if (rows.length === 0) return []
  return rows[0].values.map(r => ({
    id: r[0],
    name: r[1],
    createdAt: r[2],
    wordCount: r[3] || 0,
    masteredCount: r[4] || 0,
    dueCount: r[5] || 0
  }))
}

export function deleteDeck(id) {
  const d = ensureDB()
  d.run('DELETE FROM cards WHERE deck_id = ?', [id])
  d.run('DELETE FROM decks WHERE id = ?', [id])
  saveToStorage(d)
}

export function getDeckById(deckId) {
  const d = ensureDB()
  const rows = d.exec('SELECT id, name, created_at FROM decks WHERE id = ?', [deckId])
  if (rows.length === 0 || rows[0].values.length === 0) return null
  const r = rows[0].values[0]
  return { id: r[0], name: r[1], createdAt: r[2] }
}

// ---- Cards ----

export function importCards(deckId, cards) {
  const d = ensureDB()
  const stmt = d.prepare(`
    INSERT OR IGNORE INTO cards (id, deck_id, word, inflections, definition)
    VALUES (?, ?, ?, ?, ?)
  `)
  for (const card of cards) {
    stmt.run([generateId(), deckId, card.word, JSON.stringify(card.inflections || []), card.definition || ''])
  }
  stmt.free()
  saveToStorage(d)
}

export function getCardsByDeckId(deckId) {
  const d = ensureDB()
  const rows = d.exec(`
    SELECT id, word, inflections, definition, ef, interval, repetitions, next_review, created_at, last_review_at
    FROM cards WHERE deck_id = ?
    ORDER BY created_at DESC
  `, [deckId])
  if (rows.length === 0 || rows[0].values.length === 0) return []
  return rows[0].values.map(r => ({
    id: r[0], word: r[1],
    inflections: JSON.parse(r[2] || '[]'),
    definition: r[3], ef: r[4], interval: r[5],
    repetitions: r[6], nextReview: r[7], createdAt: r[8], lastReviewAt: r[9]
  }))
}

export function getTodayCards(deckId = null) {
  const d = ensureDB()
  const today = todayStr()
  const params = [today]
  let sql = `
    SELECT c.id, c.word, c.inflections, c.definition, c.ef, c.interval, c.repetitions,
           c.next_review, c.created_at, c.last_review_at, d.name AS deck_name
    FROM cards c
    JOIN decks d ON d.id = c.deck_id
  `
  if (deckId) {
    sql += 'WHERE c.deck_id = ? AND c.next_review <= ? '
    params.unshift(deckId)
  } else {
    sql += 'WHERE c.next_review <= ? '
  }
  sql += 'ORDER BY c.ef ASC, c.next_review ASC LIMIT 50'
  const rows = d.exec(sql, params)
  if (rows.length === 0 || rows[0].values.length === 0) return []
  return rows[0].values.map(r => ({
    id: r[0], word: r[1],
    inflections: JSON.parse(r[2] || '[]'),
    definition: r[3], ef: r[4], interval: r[5],
    repetitions: r[6], nextReview: r[7], createdAt: r[8], lastReviewAt: r[9],
    deckName: r[10]
  }))
}

export function updateCardAfterReview(cardId, { ef, interval, repetitions, nextReview }) {
  const d = ensureDB()
  d.run(`
    UPDATE cards SET ef = ?, interval = ?, repetitions = ?, next_review = ?, last_review_at = ?
    WHERE id = ?
  `, [ef, interval, repetitions, nextReview, nowStr(), cardId])
  saveToStorage(d)
}

export function getDeckStats(deckId) {
  const d = ensureDB()
  const rows = d.exec(`
    SELECT
      COUNT(*) AS total,
      SUM(CASE WHEN repetitions >= 2 THEN 1 ELSE 0 END) AS mastered,
      SUM(CASE WHEN next_review <= date('now') AND repetitions < 2 THEN 1 ELSE 0 END) AS due
    FROM cards WHERE deck_id = ?
  `, [deckId])
  if (rows.length === 0 || rows[0].values.length === 0) {
    return { total: 0, mastered: 0, due: 0 }
  }
  const r = rows[0].values[0]
  return { total: r[0] || 0, mastered: r[1] || 0, due: r[2] || 0 }
}

export function getTodayCount() {
  const d = ensureDB()
  const today = todayStr()
  const rows = d.exec('SELECT COUNT(*) FROM cards WHERE next_review <= ?', [today])
  return rows.length > 0 ? (rows[0].values[0][0] || 0) : 0
}



export function exportDB() {
  const d = ensureDB()
  return d.export()
}

export function downloadDB() {
  const data = exportDB()
  const blob = new Blob([data], { type: 'application/x-sqlite3' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'vocab_backup_' + new Date().toISOString().slice(0, 10) + '.sqlite'
  a.click()
  URL.revokeObjectURL(url)
}

// ---- Review Logs ----

export function addReviewLog(cardId, quality, efBefore, efAfter) {
  const d = ensureDB()
  d.run('INSERT INTO review_logs (id, card_id, quality, ef_before, ef_after) VALUES (?, ?, ?, ?, ?)',
    [generateId(), cardId, quality, efBefore, efAfter])
  saveToStorage(d)
}
