<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '../stores/useAppStore'
import { ArrowRight, BookOpen, CheckSquare, GraduationCap, Loader, Play, Plus, Search, Sparkles, Volume2 } from 'lucide-vue-next'
import { speakWord } from '../platform/tts.js'
import BottomNav from '../components/BottomNav.vue'

const router = useRouter()
const store = useAppStore()

onMounted(() => {
  store.refreshAll()
})

const ttsWord = ref('')
const ttsState = ref('idle')

const ttsError = ref('')
const streakDays = computed(() => store.learningStats?.streakDays || 0)
const totalCards = computed(() => store.decks.reduce((sum, deck) => sum + getDeckTotal(deck), 0))
const masteredCards = computed(() => store.decks.reduce((sum, deck) => sum + getDeckMastered(deck), 0))
const greeting = computed(() => {
  const hour = new Date().getHours()
  if (hour < 6) return '夜深了'
  if (hour < 12) return '早上好'
  if (hour < 18) return '下午好'
  return '晚上好'
})

function testTTS() {
  const word = ttsWord.value.trim()
  if (!word) return
  ttsError.value = ''
  ttsState.value = 'loading'
  speakWord(word, {
    onStateChange: state => { ttsState.value = state }
  }).catch((err) => {
    ttsState.value = 'unavailable'
    ttsError.value = err?.message || String(err)
  })
}

function goStudy(deckId) {
  router.push({ name: 'Study', query: { deckId } })
}

function goDictation() {
  router.push({ name: 'Dictation' })
}

function goDeckDetail(deckId) {
  router.push({ name: 'DeckDetail', params: { id: deckId } })
}

function goImport() {
  router.push({ name: 'Import' })
}

function goLibrary() {
  router.push({ name: 'Library' })
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
          <p class="home-greeting">{{ greeting }}</p>
          <h1 class="home-title">Recall</h1>
          <p class="home-subtitle">今天先完成最重要的复习</p>
        </div>
        <button class="home-round-action" title="导入" @click="goImport">
          <Plus class="h-5 w-5" />
        </button>
      </header>

      <section class="home-focus-card">
        <div class="home-focus-copy">
          <div class="home-focus-kicker">
            <CheckSquare class="h-4 w-4" />
            今日复习
          </div>
          <p class="home-focus-lead">先完成最小一组，后面的学习会轻很多。</p>
          <div class="home-focus-number-row">
            <span class="home-focus-number">{{ store.todayCount }}</span>
            <span class="home-focus-unit">张卡片待复习</span>
          </div>
          <p class="home-focus-time">预计 {{ Math.max(1, Math.ceil(store.todayCount / 6)) }} 分钟完成</p>
        </div>
        <div class="home-card-illustration" aria-hidden="true">
          <div class="home-card-sheet home-card-sheet-back" />
          <div class="home-card-sheet home-card-sheet-front">
            <span />
            <span />
            <span />
          </div>
          <Sparkles class="home-card-sparkle h-5 w-5" />
        </div>
        <div class="home-focus-actions">
          <button class="home-primary-action" @click="goStudy()">
            {{ store.todayCount === 0 ? '自由练习' : '开始学习' }}
            <ArrowRight class="h-4 w-4" />
          </button>
          <button class="home-audio-action" @click="goDictation">
            <Volume2 class="h-5 w-5" />
          </button>
        </div>
      </section>

      <section class="home-stat-strip">
        <div>
          <strong>{{ streakDays }}</strong>
          <span>连续天数</span>
        </div>
        <div>
          <strong>{{ masteredCards }}</strong>
          <span>已掌握</span>
        </div>
        <div>
          <strong>{{ totalCards }}</strong>
          <span>总卡片</span>
        </div>
      </section>

      <section class="mb-4">
        <div class="section-title-row">
          <h2 class="section-title">我的知识库</h2>
          <button
            class="flex items-center gap-1 text-xs font-bold text-slate-400"
            @click="goLibrary"
          >
            查看全部
            <ArrowRight class="h-3.5 w-3.5" />
          </button>
        </div>

        <div v-if="store.decks.length > 0" class="grid gap-3">
          <button
            v-for="deck in store.sortedDecks.slice(0, 3)"
            :key="deck.id"
            class="card-list-row flex w-full items-center gap-3 px-4 py-3 text-left"
            @click="goDeckDetail(deck.id)"
          >
            <div class="deck-gem grid h-12 w-12 shrink-0 place-items-center rounded-[18px] bg-gradient-to-br from-blue-500 to-cyan-400 text-white">
              <BookOpen class="h-5 w-5" />
            </div>
            <div class="min-w-0 flex-1">
              <div class="flex items-center justify-between gap-3">
                <h3 class="truncate text-sm font-black text-ink">{{ deck.name }}</h3>
                <span class="text-xs font-bold text-slate-400">{{ getTotalMasteredRatio(deck) }}%</span>
              </div>
              <p class="mt-1 text-xs font-medium text-slate-400">{{ getDeckMastered(deck) }} / {{ getDeckTotal(deck) }}</p>
              <div class="mt-2 flex items-center gap-3">
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

      <section class="home-learning-section mb-4">
        <div class="section-title-row">
          <div>
            <h2 class="section-title">学习方式</h2>
            <p class="page-copy mt-1 text-xs">按你喜欢的节奏开始</p>
          </div>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <button
            class="soft-panel action-tile flex items-center gap-3 p-4 text-left"
            @click="goDictation"
          >
            <span class="icon-well bg-blue-50 text-blue-600">
              <Volume2 class="h-5 w-5" />
            </span>
            <span class="min-w-0">
              <span class="block text-sm font-black text-ink">听写模式</span>
              <span class="mt-1 block text-xs muted">边听边写</span>
            </span>
          </button>
          <button
            class="soft-panel action-tile flex items-center gap-3 p-4 text-left"
            @click="goStudy()"
          >
            <span class="icon-well bg-emerald-50 text-emerald-600">
              <GraduationCap class="h-5 w-5" />
            </span>
            <span class="min-w-0">
              <span class="block text-sm font-black text-ink">闪卡学习</span>
              <span class="mt-1 block text-xs muted">智能复习</span>
            </span>
          </button>
        </div>
      </section>

      <section class="home-pronounce-card p-4">
        <div class="mb-3 flex items-center gap-2 text-sm font-black text-ink">
          <Volume2 class="h-4 w-4 text-blue-500" />
          快速发音测试
        </div>
        <div class="flex gap-2">
          <div class="relative flex-1">
            <Search class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-300" />
            <input
              v-model="ttsWord"
              type="text"
              placeholder="输入单词"
              class="input-soft h-12 w-full pl-9 pr-3 text-sm text-ink outline-none focus:border-blue-300 focus:bg-white"
              @keyup.enter="testTTS"
            />
          </div>
          <button
            class="blue-gradient flex h-12 w-12 items-center justify-center rounded-[18px] text-white shadow-[0_12px_22px_rgba(53,100,255,0.22)] disabled:opacity-50"
            :disabled="ttsState === 'loading' || !ttsWord.trim()"
            @click="testTTS"
            title="播放发音"
          >
            <Loader v-if="ttsState === 'loading'" class="h-4 w-4 animate-spin" />
            <Play v-else class="h-4 w-4" />
          </button>
        </div>
        <p v-if="ttsState === 'unavailable'" class="mt-2 text-xs text-red-400">发音不可用</p>
        <p v-if="ttsError" class="mt-1 break-all text-xs muted">{{ ttsError }}</p>
        <p v-else-if="ttsState === 'playing'" class="mt-2 text-xs text-emerald-500">正在播放...</p>
      </section>
    </div>
    <BottomNav />
  </div>
</template>
