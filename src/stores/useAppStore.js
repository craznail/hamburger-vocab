import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import * as deckApi from '../api/deck'
import * as cardApi from '../api/card'
import * as studyApi from '../api/study'
import * as importApi from '../api/import'

export const useAppStore = defineStore('app', () => {
  const decks = ref([])
  const todayCount = ref(0)
  const initialized = ref(false)

  async function init() {
    try {
      await deckApi.initDB()
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
    decks.value = await deckApi.getDecks()
  }

  async function refreshTodayCount() {
    todayCount.value = await deckApi.getTodayCount()
  }

  async function importFile(fileName, text) {
    const deckName = fileName.replace(/\.txt$/i, '')

    try {
      const result = await importApi.importFromText(deckName, text)
      await refreshDecks()
      await refreshTodayCount()
      return { success: true, deckId: result.deckId, deckName: result.deckName, count: result.count, result: result.parseResult }
    } catch (e) {
      return { success: false, error: e.toString() }
    }
  }

  async function removeDeck(deckId) {
    await deckApi.deleteDeck(deckId)
    await refreshDecks()
  }

  async function getDeckStats(deckId) {
    return await deckApi.getDeckStats(deckId)
  }

  async function getDeckInfo(deckId) {
    return await deckApi.getDeckById(deckId)
  }

  async function getCardsForDeck(deckId) {
    const cards = await cardApi.getCardsByDeckId(deckId)
    return cards.map(c => ({
      ...c,
      inflections: parseInflections(c.inflections)
    }))
  }

  async function getTodayLearningCards(deckId = null) {
    const cards = await cardApi.getTodayCards(deckId)
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

  async function rateCard(cardId, quality) {
    const result = await studyApi.rateCard(cardId, quality)
    await refreshTodayCount()
    return result
  }

  const sortedDecks = computed(() => {
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
