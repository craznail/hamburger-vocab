import { invoke } from '@tauri-apps/api/core'

export async function importCards(deckId, cards) {
  await invoke('import_cards', { deckId, cards })
}

export async function getCardsByDeckId(deckId) {
  return await invoke('get_cards_by_deck_id', { deckId })
}

export async function getTodayCards(deckId = null) {
  return await invoke('get_today_cards', { deckId: deckId || null })
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
