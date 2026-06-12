<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '../stores/useAppStore'
import { ArrowRight, BookOpen, Brain, CheckSquare, Flame, GraduationCap, Loader, Play, Plus, Search, Sparkles, Volume2 } from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
import { speakWord } from '../platform/tts.js'
import BottomNav from '../components/BottomNav.vue'

const router = useRouter()
const store = useAppStore()

const ttsWord = ref('')
const ttsState = ref('idle')

const ttsError = ref('')

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
  <div class="app-page flex min-h-screen flex-col">
    <NavBar :showBack="false">
      <template #left>
        <div class="flex min-w-0 items-center gap-3">
          <div class="brand-mark">
            <Brain class="h-6 w-6" />
          </div>
          <div class="min-w-0">
            <h1 class="text-xl font-black leading-none text-ink">Recall</h1>
            <p class="mt-1 truncate text-xs font-medium text-slate-400">把知识变成长久记忆</p>
          </div>
        </div>
      </template>
      <template #right>
        <button class="flex h-10 w-10 items-center justify-center rounded-lg border border-blue-100 bg-blue-50 text-blue-600 shadow-sm" title="导入" @click="goImport">
          <Plus class="h-5 w-5" />
        </button>
      </template>
    </NavBar>

    <div class="flex-1 px-5 pb-8 pt-4">
      <div class="mb-4">
        <p class="section-kicker">TODAY</p>
        <h2 class="mt-1 text-2xl font-black text-ink">晚上好，继续保持</h2>
      </div>

      <section class="home-hero p-4 text-white">
        <div class="grid grid-cols-[minmax(0,1fr)_104px] gap-4">
          <div class="min-w-0">
            <div class="flex items-center gap-2 text-sm font-semibold text-blue-100">
              <CheckSquare class="h-4 w-4" />
              今日待复习
            </div>
            <div class="mt-3 flex items-end gap-2">
              <span class="text-5xl font-black leading-none">{{ store.todayCount }}</span>
              <span class="pb-1 text-sm font-semibold text-blue-100">张卡片</span>
            </div>
            <p class="mt-2 text-xs text-blue-100/90">预计 {{ Math.max(1, Math.ceil(store.todayCount / 6)) }} 分钟完成</p>
          </div>
          <div class="study-buddy">
            <div class="buddy-head">
              <span />
            </div>
            <div class="buddy-book" />
            <Sparkles class="absolute bottom-4 right-3 h-5 w-5 text-amber-300" />
          </div>
        </div>
        <button
          class="primary-action mt-4 w-full text-sm"
          :disabled="store.todayCount === 0"
          @click="goStudy()"
        >
          {{ store.todayCount === 0 ? '今日任务已完成' : '开始今日复习' }}
          <ArrowRight class="h-4 w-4" />
        </button>
      </section>

      <section class="mt-3 flex items-center justify-between border-b border-slate-200 px-1 pb-4 pt-1">
        <div class="flex items-center gap-2">
          <Flame class="h-5 w-5 text-orange-500" />
          <span class="text-sm font-bold text-ink">连续学习 12 天</span>
        </div>
        <span class="text-xs font-medium text-slate-400">{{ store.decks.length }} 个知识库</span>
      </section>

      <section class="mt-5">
        <div class="mb-3">
          <p class="section-kicker">QUICK START</p>
          <h2 class="mt-1 text-base font-black text-ink">选择学习方式</h2>
        </div>
        <div class="grid grid-cols-2 gap-3">
        <button
          class="soft-panel action-tile flex items-center gap-3 p-3 text-left"
          @click="goDictation"
        >
          <span class="icon-well bg-blue-50 text-blue-600">
            <Volume2 class="h-5 w-5" />
          </span>
          <span class="min-w-0">
            <span class="block text-sm font-bold text-ink">听写模式</span>
            <span class="mt-1 block text-xs muted">边听边写</span>
          </span>
        </button>
        <button
          class="soft-panel action-tile flex items-center gap-3 p-3 text-left"
          @click="goStudy()"
        >
          <span class="icon-well bg-emerald-50 text-emerald-600">
            <GraduationCap class="h-5 w-5" />
          </span>
          <span class="min-w-0">
            <span class="block text-sm font-bold text-ink">闪卡学习</span>
            <span class="mt-1 block text-xs muted">智能复习</span>
          </span>
        </button>
        </div>
      </section>

      <section class="mt-6">
        <div class="mb-3 flex items-center justify-between">
          <div>
            <p class="section-kicker">LIBRARY</p>
            <h2 class="mt-1 text-base font-black text-ink">我的知识库</h2>
          </div>
          <button
            class="flex items-center gap-1 text-xs font-semibold text-blue-600"
            @click="goLibrary"
          >
            查看全部
            <ArrowRight class="h-3.5 w-3.5" />
          </button>
        </div>
        <div v-if="store.decks.length > 0" class="space-y-3">
          <div
            v-for="deck in store.sortedDecks.slice(0, 3)"
            :key="deck.id"
            class="soft-panel interactive-surface cursor-pointer p-4"
            @click="goDeckDetail(deck.id)"
          >
            <div class="flex items-center gap-3">
              <div class="deck-gem flex h-12 w-12 items-center justify-center rounded-lg bg-gradient-to-br from-blue-500 to-cyan-400 text-white shadow-sm">
                <BookOpen class="h-6 w-6" />
              </div>
              <div class="min-w-0 flex-1">
                <div class="flex items-center justify-between gap-3">
                  <h3 class="truncate text-sm font-bold text-ink">{{ deck.name }}</h3>
                  <ArrowRight class="h-4 w-4 shrink-0 text-slate-300" />
                </div>
                <div class="mt-1 flex items-center gap-2 text-xs muted">
                  <span>{{ getDeckMastered(deck) }} / {{ getDeckTotal(deck) }}</span>
                  <span v-if="getDeckDue(deck) > 0" class="text-amber-500">今日 {{ getDeckDue(deck) }}</span>
                </div>
                <div class="mt-2 flex items-center gap-2">
                  <div class="progress-track h-1.5 flex-1">
                    <div
                      class="progress-fill transition-all"
                      :style="{ width: `${getTotalMasteredRatio(deck)}%` }"
                    />
                  </div>
                  <span class="w-9 text-right text-xs font-semibold text-blue-500">{{ getTotalMasteredRatio(deck) }}%</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-else class="soft-panel p-7 text-center">
          <BookOpen class="mx-auto mb-3 h-12 w-12 text-blue-200" />
          <p class="mb-1 text-sm font-semibold text-slate-500">还没有知识库</p>
          <p class="text-xs muted">导入文本文件后就能开始学习</p>
        </div>
      </section>

      <section class="soft-panel mt-5 p-4">
        <div class="mb-3 flex items-center gap-2 text-sm font-bold text-ink">
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
              class="h-11 w-full rounded-lg border border-slate-200 bg-slate-50 pl-9 pr-3 text-sm text-ink outline-none transition-colors focus:border-blue-300 focus:bg-white"
              @keyup.enter="testTTS"
            />
          </div>
          <button
            class="flex h-11 w-11 items-center justify-center rounded-lg bg-blue-600 text-white shadow-sm transition-colors hover:bg-blue-700 disabled:opacity-50"
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
