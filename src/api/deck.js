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
