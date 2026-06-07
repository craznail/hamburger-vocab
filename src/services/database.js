import { invoke } from '@tauri-apps/api/core'

export async function initDB() {
  try {
    await invoke('init_db')
  } catch (e) {
    console.error('Failed to initialize database:', e)
    throw e
  }
}

export async function createDeck(name) {
  return await invoke('create_deck', { name })
}

export async function getDecks() {
  return await invoke('get_decks')
}

export async function deleteDeck(id) {
  await invoke('delete_deck', { deckId: id })
}

export async function getDeckById(deckId) {
  return await invoke('get_deck_by_id', { deckId })
}

export async function getDeckStats(deckId) {
  return await invoke('get_deck_stats', { deckId })
}

export async function getTodayCount() {
  return await invoke('get_today_count')
}

export async function importCards(deckId, cards) {
  await invoke('import_cards', { deckId, cards })
}

export async function getCardsByDeckId(deckId) {
  return await invoke('get_cards_by_deck_id', { deckId })
}

export async function getTodayCards(deckId = null) {
  return await invoke('get_today_cards', { deckId: deckId || null })
}

export async function updateCardAfterReview(cardId, { ef, interval, repetitions, nextReview }) {
  await invoke('update_card_after_review', { cardId, ef, interval, repetitions, nextReview })
}

export async function addReviewLog(cardId, quality, efBefore, efAfter) {
  await invoke('add_review_log', { cardId, quality, efBefore, efAfter })
}

export async function getExportDbPath() {
  return await invoke('export_db_path')
}

export async function downloadDB() {
  const { save } = await import('@tauri-apps/plugin-dialog')
  const { copyFile } = await import('@tauri-apps/plugin-fs')

  const dbPath = await getExportDbPath()
  const savePath = await save({
    defaultPath: `vocab_backup_${new Date().toISOString().slice(0, 10)}.sqlite`,
    filters: [{ name: 'SQLite Database', extensions: ['sqlite'] }]
  })
  if (savePath) {
    await copyFile(dbPath, savePath)
  }
}

export async function readTxtFile(path) {
  const { readTextFile } = await import("@tauri-apps/plugin-fs")
  
  // Timeout to prevent UI hanging on Android
  const TIMEOUT_MS = 15000
  const timeout = new Promise((_, reject) =>
    setTimeout(() => reject(new Error(`读取文件超时 (${path.substring(0, 50)}...)`)), TIMEOUT_MS)
  )

  console.log('[readTxtFile] path:', path)
  const result = await Promise.race([readTextFile(path), timeout])
  console.log('[readTxtFile] success, length:', result.length)
  return result
}

