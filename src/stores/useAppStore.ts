import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import * as deckApi from '../api/deck'
import * as cardApi from '../api/card'
import * as studyApi from '../api/study'
import * as importApi from '../api/import'
import type { Deck } from '../types/generated/Deck'
import type { DeckInfo } from '../types/generated/DeckInfo'
import type { DeckStats } from '../types/generated/DeckStats'
import type { Card } from '../types/generated/Card'
import type { TodayCard } from '../types/generated/TodayCard'
import type { RateResult } from '../types/generated/RateResult'
import type { LearningStats } from '../types/generated/LearningStats'

interface ImportResult {
  success: boolean
  deckId?: string
  deckName?: string
  count?: number
  result?: unknown
  error?: string
}

interface LearningCard extends TodayCard {
  inflections: string[]
}

export const useAppStore = defineStore('app', () => {
  const decks = ref<Deck[]>([])
  const todayCount = ref<number>(0)
  const learningStats = ref<LearningStats | null>(null)
  const initialized = ref<boolean>(false)

  async function init(): Promise<void> {
    try {
      await deckApi.initDB()
      await Promise.all([refreshDecks(), refreshTodayCount(), refreshLearningStats()])
    } catch (e) {
      console.error('数据库初始化失败:', e)
      decks.value = []
      todayCount.value = 0
      learningStats.value = null
    }
    initialized.value = true
  }

  async function refreshDecks(): Promise<void> {
    decks.value = await deckApi.getDecks()
  }

  async function refreshTodayCount(): Promise<void> {
    todayCount.value = await deckApi.getTodayCount()
  }

  async function refreshLearningStats(): Promise<void> {
    learningStats.value = await studyApi.getLearningStats()
  }

  async function importFile(fileName: string, text: string): Promise<ImportResult> {
    const deckName = fileName.replace(/\.txt$/i, '')

    try {
      const result = await importApi.importFromText(deckName, text)
      await Promise.all([refreshDecks(), refreshTodayCount(), refreshLearningStats()])
      return { success: true, deckId: result.deckId, deckName: result.deckName, count: result.count, result: result.parseResult }
    } catch (e) {
      return { success: false, error: String(e) }
    }
  }

  async function removeDeck(deckId: string): Promise<void> {
    await deckApi.deleteDeck(deckId)
    await Promise.all([refreshDecks(), refreshTodayCount(), refreshLearningStats()])
  }

  async function getDeckStats(deckId: string): Promise<DeckStats> {
    return await deckApi.getDeckStats(deckId)
  }

  async function getDeckInfo(deckId: string): Promise<DeckInfo | null> {
    return await deckApi.getDeckById(deckId)
  }

  async function getCardsForDeck(deckId: string): Promise<LearningCard[]> {
    const cards = await cardApi.getCardsByDeckId(deckId)
    return cards.map(c => ({
      ...c,
      inflections: parseInflections(c.inflections),
    }))
  }

  async function getTodayLearningCards(deckId: string | null = null): Promise<LearningCard[]> {
    const cards = await cardApi.getTodayCards(deckId)
    return normalizeLearningCards(cards)
  }

  async function getPracticeCards(deckId: string | null = null): Promise<LearningCard[]> {
    const cards = await cardApi.getPracticeCards(deckId)
    return normalizeLearningCards(cards)
  }

  function normalizeLearningCards(cards: TodayCard[]): LearningCard[] {
    return cards.map(c => ({
      ...c,
      inflections: parseInflections(c.inflections),
    }))
  }

  function parseInflections(inflections: string | null): string[] {
    if (!inflections) return []
    try {
      return JSON.parse(inflections) as string[]
    } catch {
      return []
    }
  }

  async function rateCard(
    cardId: string,
    quality: number,
    durationSeconds?: number,
  ): Promise<RateResult> {
    const result = await studyApi.rateCard(cardId, quality, durationSeconds ?? 0)
    if (todayCount.value > 0) {
      todayCount.value -= 1
    }
    return result
  }

  async function ratePracticeCard(
    cardId: string,
    quality: number,
    durationSeconds?: number,
  ): Promise<void> {
    await studyApi.ratePracticeCard(cardId, quality, durationSeconds ?? 0)
  }

  /** Full refresh — call after completing a study session or when entering a data page. */
  async function refreshAll(): Promise<void> {
    await Promise.all([refreshDecks(), refreshTodayCount(), refreshLearningStats()])
  }

  const sortedDecks = computed<Deck[]>(() => {
    return [...decks.value].sort((a, b) => {
      const aDue = a.due_count > 0 ? 0 : 1
      const bDue = b.due_count > 0 ? 0 : 1
      if (aDue !== bDue) return aDue - bDue
      return a.name.localeCompare(b.name)
    })
  })

  return {
    decks,
    todayCount,
    learningStats,
    initialized,
    sortedDecks,
    init,
    refreshDecks,
    refreshTodayCount,
    refreshLearningStats,
    refreshAll,
    importFile,
    removeDeck,
    getDeckStats,
    getDeckInfo,
    getCardsForDeck,
    getTodayLearningCards,
    getPracticeCards,
    rateCard,
    ratePracticeCard,
  }
})
