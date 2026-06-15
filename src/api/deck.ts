import { invoke } from '@tauri-apps/api/core'
import type { Deck } from '../types/generated/Deck'
import type { DeckInfo } from '../types/generated/DeckInfo'
import type { DeckStats } from '../types/generated/DeckStats'

export async function initDB(): Promise<void> {
  await invoke('init_db')
}

export async function createDeck(name: string): Promise<string> {
  return await invoke<string>('create_deck', { name })
}

export async function getDecks(): Promise<Deck[]> {
  return await invoke<Deck[]>('get_decks')
}

export async function deleteDeck(id: string): Promise<void> {
  await invoke('delete_deck', { deckId: id })
}

export async function getDeckById(deckId: string): Promise<DeckInfo | null> {
  return await invoke<DeckInfo | null>('get_deck_by_id', { deckId })
}

export async function getDeckStats(deckId: string): Promise<DeckStats> {
  return await invoke<DeckStats>('get_deck_stats', { deckId })
}

export async function getTodayCount(): Promise<number> {
  return await invoke<number>('get_today_count')
}
