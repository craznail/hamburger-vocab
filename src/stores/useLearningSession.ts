import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useLearningStore, type LearningSection } from './useLearningStore'

export interface SessionItem {
  type: string
  title: string
  routeName: string
  query?: Record<string, string>
}

export interface SessionResult {
  type: string
  title: string
  quality: 'again' | 'hard' | 'good' | 'easy'
  timestamp: number
}

export const useLearningSession = defineStore('learningSession', () => {
  const learningStore = useLearningStore()

  const active = ref(false)
  const index = ref(0)
  const queue = ref<SessionItem[]>([])
  const results = ref<SessionResult[]>([])

  const current = computed(() => queue.value[index.value] || null)
  const isFinished = computed(() => active.value && index.value >= queue.value.length)

  function getRecentScore(type: string) {
    const recent = results.value.filter(r => r.type === type).slice(-5)
    if (recent.length === 0) return 0

    let score = 0
    recent.forEach(r => {
      if (r.quality === 'again') score += 2
      if (r.quality === 'hard') score += 1
      if (r.quality === 'easy') score -= 1
    })
    return score
  }

  function buildQueue(): SessionItem[] {
    const sections = learningStore.sections
      .filter((s: LearningSection) => s.enabled && s.count > 0)

    const baseWeight: Record<string, number> = {
      word_review: 1,
      dictation: 1.2,
      error_review: 2.8,
      free_practice: 0.5,
    }

    const items: SessionItem[] = []

    const enriched = sections.map(section => {
      const base = baseWeight[section.type] || 1
      const recent = getRecentScore(section.type)

      // error boost
      const errorBoost = section.type === 'error_review' && section.count > 0 ? 1.5 : 1

      // fatigue penalty (too many items already done)
      const fatigue = Math.min(results.value.length * 0.02, 0.6)

      const score = base * errorBoost + recent - fatigue

      return { section, score }
    })

    enriched.sort((a, b) => b.score - a.score)

    enriched.forEach(({ section }) => {
      const limit = Math.min(section.count, 20)
      for (let i = 0; i < limit; i++) {
        items.push({
          type: section.type,
          title: section.title,
          routeName: section.routeName,
          query: section.query,
        })
      }
    })

    return items
  }

  function start() {
    queue.value = buildQueue()
    index.value = 0
    results.value = []
    active.value = true
  }

  function record(quality: SessionResult['quality']) {
    const item = current.value
    if (!item) return

    results.value.push({
      type: item.type,
      title: item.title,
      quality,
      timestamp: Date.now(),
    })
  }

  function next() {
    if (index.value < queue.value.length) {
      index.value++
    }

    if (index.value >= queue.value.length) {
      active.value = false
    }
  }

  function reset() {
    active.value = false
    index.value = 0
    queue.value = []
    results.value = []
  }

  return {
    active,
    index,
    queue,
    results,
    current,
    isFinished,
    start,
    next,
    record,
    reset,
  }
})