import { invoke } from '@tauri-apps/api/core'
import type { Card } from '../types/generated/Card'
import type { TodayCard } from '../types/generated/TodayCard'

export async function importCards(
  deckId: string,
  cards: Array<{ word: string; inflections: string[]; definition: string }>,
): Promise<void> {
  await invoke('import_cards', { deckId, cards })
}

export async function getCardsByDeckId(deckId: string): Promise<Card[]> {
  return await invoke<Card[]>('get_cards_by_deck_id', { deckId })
}

export async function getTodayCards(deckId: string | null = null): Promise<TodayCard[]> {
  return await invoke<TodayCard[]>('get_today_cards', { deckId: deckId || null })
}

export async function getPracticeCards(deckId: string | null = null): Promise<TodayCard[]> {
  return await invoke<TodayCard[]>('get_practice_cards', { deckId: deckId || null })
}

export async function getExportDbPath(): Promise<string> {
  return await invoke<string>('export_db_path')
}

export async function downloadDB(): Promise<void> {
  const { save } = await import('@tauri-apps/plugin-dialog')
  const { copyFile } = await import('@tauri-apps/plugin-fs')

  const dbPath = await getExportDbPath()
  const savePath = await save({
    defaultPath: `vocab_backup_${new Date().toISOString().slice(0, 10)}.sqlite`,
    filters: [{ name: 'SQLite Database', extensions: ['sqlite'] }],
  })
  if (savePath) {
    await copyFile(dbPath, savePath)
  }
}
