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
  const previewMode = ref(false)

  const previewDecks = [
    { id: 1, name: 'Redis 核心知识', word_count: 500, mastered_count: 235, due_count: 15 },
    { id: 2, name: '英语四级词汇', word_count: 2000, mastered_count: 812, due_count: 8 },
    { id: 3, name: '系统设计基础', word_count: 300, mastered_count: 152, due_count: 12 },
    { id: 4, name: '计算机网络', word_count: 200, mastered_count: 82, due_count: 9 },
    { id: 5, name: '产品经理知识体系', word_count: 250, mastered_count: 108, due_count: 6 }
  ]

  const previewCards = [
    {
      id: 101,
      word: 'Redis 中用于设置键过期时间的命令是？',
      inflections: ['EXPIRE key seconds'],
      definition: 'EXPIRE 用于给键设置秒级过期时间，TTL 可查询剩余时间。',
      ef: 2.5,
      interval: 1,
      repetitions: 0
    },
    {
      id: 102,
      word: 'abandon',
      inflections: ['[əˈbændən]'],
      definition: 'v. 放弃；抛弃；停止支持',
      ef: 2.5,
      interval: 1,
      repetitions: 0
    },
    {
      id: 103,
      word: 'TCP 三次握手的目的是什么？',
      inflections: ['SYN', 'SYN-ACK', 'ACK'],
      definition: '确认双方收发能力，并协商初始序列号，建立可靠连接。',
      ef: 2.5,
      interval: 1,
      repetitions: 0
    }
  ]

  async function init() {
    try {
      await deckApi.initDB()
      await refreshDecks()
      await refreshTodayCount()
    } catch (e) {
      console.warn('数据库初始化失败（预览模式时正常）:', e)
      previewMode.value = true
      decks.value = previewDecks
      todayCount.value = 32
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
    if (previewMode.value) {
      const deck = previewDecks.find(d => String(d.id) === String(deckId))
      return {
        total: deck?.word_count || 0,
        mastered: deck?.mastered_count || 0,
        due: deck?.due_count || 0
      }
    }
    return await deckApi.getDeckStats(deckId)
  }

  async function getDeckInfo(deckId) {
    if (previewMode.value) {
      return previewDecks.find(d => String(d.id) === String(deckId)) || null
    }
    return await deckApi.getDeckById(deckId)
  }

  async function getCardsForDeck(deckId) {
    if (previewMode.value) return previewCards
    const cards = await cardApi.getCardsByDeckId(deckId)
    return cards.map(c => ({
      ...c,
      inflections: parseInflections(c.inflections)
    }))
  }

  async function getTodayLearningCards(deckId = null) {
    if (previewMode.value) return previewCards
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
    previewMode,
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
