<script setup>
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '../stores/useAppStore'
import { useErrorNotebookStore } from '../stores/useErrorNotebookStore'
import {
  BookOpen,
  CalendarDays,
  CheckSquare,
  ChevronRight,
  CirclePlay,
  Dices,
  Flame,
  GraduationCap,
  Headphones,
  NotebookPen,
} from 'lucide-vue-next'
import BottomNav from '../components/BottomNav.vue'

const router = useRouter()
const store = useAppStore()
const errorStore = useErrorNotebookStore()

onMounted(() => {
  store.refreshAll()
  void errorStore.ensureFresh()
})

const streakDays = computed(() => store.learningStats?.streakDays || 0)
const masteredCards = computed(() => store.decks.reduce((sum, deck) => sum + getDeckMastered(deck), 0))
const dueMistakes = computed(() => errorStore.dueCount || 0)
const topDecks = computed(() => store.sortedDecks.slice(0, 2))

function goStudy({ deckId = undefined, practice = false } = {}) {
  router.push({
    name: 'Study',
    query: {
      ...(deckId ? { deckId } : {}),
      ...(practice ? { mode: 'practice' } : {}),
    },
  })
}

function goDictation() {
  router.push({ name: 'Dictation' })
}

function goDeckDetail(deckId) {
  router.push({ name: 'DeckDetail', params: { id: deckId } })
}

function goLibrary() {
  router.push({ name: 'Library' })
}

function goCalendar() {
  router.push({ name: 'Stats' })
}

function getTotalMasteredRatio(deck) {
  const total = deck.wordCount || deck.word_count || deck.total || 0
  const mastered = deck.masteredCount || deck.mastered_count || deck.mastered || 0
  if (total === 0) return 0
  return Math.round((mastered / total) * 100)
}

function getDeckTotal(deck) {
  return deck.wordCount || deck.word_count || deck.total || 0
}

function getDeckMastered(deck) {
  return deck.masteredCount || deck.mastered_count || deck.mastered || 0
}

function getDeckDue(deck) {
  return deck.dueCount || deck.due_count || deck.due || 0
}
</script>

<template>
  <div class="app-page home-page flex min-h-screen flex-col">
    <div class="home-shell">
      <header class="home-topbar">
        <div class="min-w-0">
          <h1 class="home-title">Recall</h1>
          <p class="home-subtitle">今天先完成最重要的复习</p>
        </div>
        <button class="home-calendar-action" title="复习日历" @click="goCalendar">
          <CalendarDays class="h-4.5 w-4.5" />
          <span>复习日历</span>
        </button>
      </header>

      <section class="home-focus-card mb-2">
        <div class="home-focus-hero">
          <div class="home-focus-copy">
            <div class="home-focus-kicker">
              <CheckSquare class="h-4 w-4" />
              今日复习
            </div>
            <div class="home-focus-number-row">
              <span class="home-focus-number">{{ store.todayCount }}</span>
            </div>
            <p class="home-focus-unit">张卡片待复习</p>
          </div>
          <div class="home-card-illustration" aria-hidden="true">
            <div class="home-card-grid" />
            <div class="home-card-stack home-card-stack-back" />
            <div class="home-card-stack home-card-stack-mid" />
            <div class="home-card-stack home-card-stack-front">
              <strong>abandon</strong>
              <span />
              <span />
              <span />
            </div>
            <div class="home-card-ring" />
            <div class="home-card-pencil" />
          </div>
        </div>

        <div class="home-focus-actions">
          <button class="home-primary-action" @click="goStudy()">
            <CirclePlay class="h-5 w-5" />
            {{ store.todayCount === 0 ? '自由练习' : '开始学习' }}
          </button>
          <button class="home-secondary-action" @click="goStudy({ practice: true })">
            <Dices class="h-5 w-5" />
            自由练习
          </button>
        </div>

        <div class="home-stat-strip">
          <div class="home-stat-item">
            <Flame class="h-6 w-6 text-emerald-500" />
            <div>
              <strong>{{ streakDays }}</strong>
              <span>坚持学习</span>
            </div>
          </div>
          <div class="home-stat-item">
            <GraduationCap class="h-6 w-6 text-blue-500" />
            <div>
              <strong>{{ masteredCards }}</strong>
              <span>熟悉单词</span>
            </div>
          </div>
          <div class="home-stat-item">
            <NotebookPen class="h-6 w-6 text-amber-500" />
            <div>
              <strong>{{ dueMistakes }}</strong>
              <span>待巩固</span>
            </div>
          </div>
        </div>
      </section>

      <section class="home-section-card mb-2">
        <div class="section-title-row">
          <h2 class="section-title">我的知识库</h2>
          <button class="home-inline-link" @click="goLibrary">
            全部知识库
            <ChevronRight class="h-4 w-4" />
          </button>
        </div>

        <div v-if="store.decks.length > 0" class="home-library-list">
          <button
            v-for="(deck, index) in topDecks"
            :key="deck.id"
            class="home-library-row"
            @click="goDeckDetail(deck.id)"
          >
            <div
              class="home-library-icon"
              :class="index === 0 ? 'home-library-icon-blue' : 'home-library-icon-green'"
            >
              <BookOpen class="h-5 w-5" />
            </div>
            <div class="min-w-0 flex-1">
              <div class="flex items-center justify-between gap-3">
                <div class="min-w-0 flex-1">
                  <h3 class="truncate text-[1.02rem] font-black text-ink">{{ deck.name }}</h3>
                  <p class="mt-1 text-sm font-medium text-slate-400">已掌握 {{ getDeckMastered(deck) }} / {{ getDeckTotal(deck) }}</p>
                </div>
                <div class="home-library-meta">
                  <span class="home-library-percent">{{ getTotalMasteredRatio(deck) }}%</span>
                  <ChevronRight class="h-4 w-4" />
                </div>
              </div>
              <div class="mt-3 flex items-center gap-3">
                <div class="progress-track h-1.5 flex-1">
                  <div class="progress-fill transition-all" :style="{ width: `${getTotalMasteredRatio(deck)}%` }" />
                </div>
                <span class="text-[11px] font-semibold text-slate-400">今日 {{ getDeckDue(deck) }}</span>
              </div>
            </div>
          </button>
        </div>

        <div v-else class="home-empty-card p-7 text-center">
          <BookOpen class="mx-auto mb-3 h-12 w-12 text-blue-200" />
          <p class="mb-1 text-sm font-semibold text-slate-500">还没有知识库</p>
          <p class="text-xs muted">导入文本文件后就能开始学习</p>
        </div>
      </section>

      <section class="home-section-card mb-2">
        <div class="section-title-row">
          <h2 class="section-title">学习方式</h2>
        </div>

        <div class="grid gap-3">
          <button
            class="home-method-row"
            @click="goStudy({ practice: true })"
          >
            <span class="home-method-icon home-method-icon-blue">
              <Dices class="h-5 w-5" />
            </span>
            <span class="min-w-0 flex-1">
              <span class="block text-[1.02rem] font-black text-ink">闪卡学习</span>
              <span class="mt-1 block text-sm muted">科学记忆，高效复习</span>
            </span>
            <ChevronRight class="h-5 w-5 text-slate-300" />
          </button>
          <button
            class="home-method-row"
            @click="goDictation"
          >
            <span class="home-method-icon home-method-icon-green">
              <Headphones class="h-5 w-5" />
            </span>
            <span class="min-w-0 flex-1">
              <span class="block text-[1.02rem] font-black text-ink">听写训练</span>
              <span class="mt-1 block text-sm muted">听音辨词，强化拼写</span>
            </span>
            <ChevronRight class="h-5 w-5 text-slate-300" />
          </button>
        </div>
      </section>
    </div>
    <BottomNav />
  </div>
</template>
