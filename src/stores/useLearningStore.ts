import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { useAppStore } from './useAppStore'
import { useErrorNotebookStore } from './useErrorNotebookStore'

export type LearningSectionType = 'word_review' | 'dictation' | 'error_review' | 'free_practice'

export interface LearningSection {
  type: LearningSectionType
  title: string
  description: string
  count: number
  unit: string
  routeName: string
  query?: Record<string, string>
  primary: boolean
  enabled: boolean
}

export const useLearningStore = defineStore('learning', () => {
  const loading = ref(false)
  const initialized = ref(false)
  const appStore = useAppStore()
  const errorStore = useErrorNotebookStore()

  const wordDueCount = computed(() => appStore.todayCount || 0)
  const errorDueCount = computed(() => errorStore.dueCount || 0)
  const dictationCount = computed(() => {
    const totalWords = appStore.decks.reduce((sum, deck) => {
      return sum + Number(deck.wordCount || deck.word_count || deck.total || 0)
    }, 0)
    return totalWords > 0 ? Math.min(12, Math.max(4, wordDueCount.value || 8)) : 0
  })
  const totalDueCount = computed(() => wordDueCount.value + errorDueCount.value)

  const sections = computed<LearningSection[]>(() => [
    {
      type: 'word_review',
      title: '单词复习',
      description: wordDueCount.value > 0 ? '按记忆计划复习到期单词' : '今日计划已完成，可进入自由练习',
      count: wordDueCount.value,
      unit: '张',
      routeName: 'WordReview',
      primary: wordDueCount.value > 0,
      enabled: appStore.decks.length > 0,
    },
    {
      type: 'dictation',
      title: '听写训练',
      description: '听音回忆拼写，强化输出记忆',
      count: dictationCount.value,
      unit: '词',
      routeName: 'Dictation',
      primary: false,
      enabled: dictationCount.value > 0,
    },
    {
      type: 'error_review',
      title: '错题复习',
      description: errorDueCount.value > 0 ? '复盘错因，把薄弱点变成长期记忆' : '暂无到期错题，可先录入或整理错题',
      count: errorDueCount.value,
      unit: '道',
      routeName: 'ErrorReview',
      primary: errorDueCount.value > 0 && wordDueCount.value === 0,
      enabled: errorStore.hasData,
    },
    {
      type: 'free_practice',
      title: '自由练习',
      description: '不推进复习计划，适合完成今日任务后加练',
      count: 0,
      unit: '',
      routeName: 'WordReview',
      query: { mode: 'practice' },
      primary: totalDueCount.value === 0 && appStore.decks.length > 0,
      enabled: appStore.decks.length > 0,
    },
  ])

  const primarySection = computed(() => {
    return sections.value.find(section => section.primary && section.enabled)
      || sections.value.find(section => section.count > 0 && section.enabled)
      || sections.value.find(section => section.enabled)
      || sections.value[0]
  })

  async function refresh(): Promise<void> {
    loading.value = true
    try {
      await Promise.all([
        appStore.refreshAll(),
        errorStore.ensureFresh(),
      ])
      initialized.value = true
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    initialized,
    wordDueCount,
    errorDueCount,
    dictationCount,
    totalDueCount,
    sections,
    primarySection,
    refresh,
  }
})
