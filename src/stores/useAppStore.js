import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import * as db from '../services/database.js'
import { parseTxtContent } from '../utils/parser.js'
import { computeNextReview } from '../utils/sm2.js'

export const useAppStore = defineStore('app', () => {
  const decks = ref([])
  const todayCount = ref(0)
  const initialized = ref(false)

  async function init() {
    try {
      await db.initDB()
      await refreshDecks()
      await refreshTodayCount()
    } catch (e) {
      console.warn('数据库初始化失败（预览模式时正常）:', e)
      decks.value = []
      todayCount.value = 0
    }
    initialized.value = true
  }

  async function refreshDecks() {
    decks.value = await db.getDecks()
  }

  async function refreshTodayCount() {
    todayCount.value = await db.getTodayCount()
  }

  function importFile(fileName, text) {
    const result = parseTxtContent(text)
    if (result.rows.length === 0) {
      return { success: false, error: '文件中没有有效的单词', result }
    }

    // Parse result.rows into the format Tauri expects
    const cards = result.rows.map(r => ({
      word: r.word,
      inflections: r.inflections || [],
      definition: r.definition || ''
    }))

    // Use filename without .txt extension as deck name
    const deckName = fileName.replace(/\.txt$/i, '')

    // Return a promise that resolves after async import
    return (async () => {
      try {
        const deckId = await db.createDeck(deckName)
        await db.importCards(deckId, cards)
        await refreshDecks()
        await refreshTodayCount()
        return { success: true, deckId, deckName, count: result.rows.length, result }
      } catch (e) {
        return { success: false, error: e.toString(), result }
      }
    })()
  }

  async function removeDeck(deckId) {
    await db.deleteDeck(deckId)
    await refreshDecks()
  }

  async function getDeckStats(deckId) {
    return await db.getDeckStats(deckId)
  }

  async function getDeckInfo(deckId) {
    return await db.getDeckById(deckId)
  }

  async function getCardsForDeck(deckId) {
    const cards = await db.getCardsByDeckId(deckId)
    // Parse inflections JSON string to array
    return cards.map(c => ({
      ...c,
      inflections: parseInflections(c.inflections)
    }))
  }

  async function getTodayLearningCards(deckId = null) {
    const cards = await db.getTodayCards(deckId)
    return cards.map(c => ({
      ...c,
      inflections: parseInflections(c.inflections)
    }))
  }

  function parseInflections(inflections) {
    if (!inflections) return []
    try {
      return JSON.parse(inflections)
    } catch {
      return []
    }
  }

  async function rateCard(cardId, quality, cardState) {
    const result = computeNextReview(quality, cardState)
    const beforeEF = cardState.ef
    await db.updateCardAfterReview(cardId, result)
    await db.addReviewLog(cardId, quality, beforeEF, result.ef)
    await refreshTodayCount()
    return result
  }

  const sortedDecks = computed(() => {
    return [...decks.value].sort((a, b) => {
      // Put decks with due cards first, then by name
      const aDue = a.due_count > 0 ? 0 : 1
      const bDue = b.due_count > 0 ? 0 : 1
      if (aDue !== bDue) return aDue - bDue
      return a.name.localeCompare(b.name)
    })
  })

  return {
    decks,
    todayCount,
    initialized,
    sortedDecks,
    init,
    refreshDecks,
    refreshTodayCount,
    importFile,
    removeDeck,
    getDeckStats,
    getDeckInfo,
    getCardsForDeck,
    getTodayLearningCards,
    rateCard
  }
})
