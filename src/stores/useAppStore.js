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
    await db.initDB()
    await refreshDecks()
    await refreshTodayCount()
    initialized.value = true
  }

  async function refreshDecks() {
    decks.value = db.getDecks()
  }

  async function refreshTodayCount() {
    todayCount.value = db.getTodayCount()
  }

  function importFile(fileName, text) {
    const result = parseTxtContent(text)
    if (result.rows.length === 0) {
      return { success: false, error: '文件中没有有效的单词', result }
    }

    // Use filename without .txt extension as deck name
    const deckName = fileName.replace(/\.txt$/i, '')
    const deckId = db.createDeck(deckName)
    db.importCards(deckId, result.rows)

    refreshDecks()
    refreshTodayCount()

    return { success: true, deckId, deckName, count: result.rows.length, result }
  }

  function removeDeck(deckId) {
    db.deleteDeck(deckId)
    refreshDecks()
  }

  function getDeckStats(deckId) {
    return db.getDeckStats(deckId)
  }

  function getDeckInfo(deckId) {
    return db.getDeckById(deckId)
  }

  function getCardsForDeck(deckId) {
    return db.getCardsByDeckId(deckId)
  }

  function getTodayLearningCards(deckId = null) {
    return db.getTodayCards(deckId)
  }

  function rateCard(cardId, quality, cardState) {
    const result = computeNextReview(quality, cardState)
    const beforeEF = cardState.ef
    db.updateCardAfterReview(cardId, result)
    db.addReviewLog(cardId, quality, beforeEF, result.ef)
    refreshTodayCount()
    return result
  }

  const sortedDecks = computed(() => {
    return [...decks.value].sort((a, b) => {
      // Put decks with due cards first, then by name
      const aDue = a.dueCount > 0 ? 0 : 1
      const bDue = b.dueCount > 0 ? 0 : 1
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
