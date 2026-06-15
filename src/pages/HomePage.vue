<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '../stores/useAppStore'
import { ArrowRight, BookOpen, Brain, CheckSquare, Flame, GraduationCap, Loader, Play, Plus, Search, Sparkles, Volume2 } from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
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
  <div class="app-page flex min-h-screen flex-col">
    <NavBar :showBack="false">
      <template #left>
        <div class="flex min-w-0 items-center gap-3">
          <div class="brand-mark">
            <Brain class="h-6 w-6" />
          </div>
          <div class="min-w-0">
            <h1 class="text-[1.65rem] font-black leading-none text-ink">Recall</h1>
            <p class="mt-1 truncate text-[12px] font-semibold text-slate-400">把任何知识变成长久记忆</p>
          </div>
        </div>
      </template>
      <template #right>
        <button class="grid h-10 w-10 place-items-center rounded-full border border-[#d9e5ff] bg-white text-blue-600 shadow-[0_10px_20px_rgba(95,126,194,0.08)]" title="导入" @click="goImport">
          <Plus class="h-5 w-5" />
        </button>
      </template>
    </NavBar>

    <div class="flex-1 px-4 pb-8 pt-4">
      <div class="mb-4 px-1">
        <h2 class="text-[2rem] font-black tracking-[-0.03em] text-ink">{{ greeting }}！</h2>
        <p class="mt-1 text-sm font-medium text-slate-400">愿你今天也有所收获</p>
      </div>

      <section class="mb-4 grid gap-4">
        <div class="glass-card relative overflow-hidden px-4 py-4">
          <div class="grid grid-cols-[minmax(0,1fr)_138px] gap-3">
            <div class="flex min-w-0 flex-col justify-between">
              <div>
                <div class="mb-3 flex items-center gap-2 text-sm font-bold text-[#ff8d46]">
                  <Flame class="h-4 w-4" />
                  连续学习
                </div>
                <div class="flex items-end gap-2">
                  <span class="text-5xl font-black leading-none text-ink">{{ streakDays }}</span>
                  <span class="pb-1 text-base font-bold text-ink-soft">天</span>
                </div>
                <p class="mt-2 text-xs leading-5 text-slate-400">{{ streakDays ? `最长连续 ${store.learningStats?.longestStreak || streakDays} 天` : '完成一次复习，开始记录连续学习' }}</p>
              </div>
            </div>
            <div class="study-buddy min-h-[152px]">
              <div class="buddy-head">
                <span />
              </div>
              <div class="buddy-book" />
              <div class="absolute right-2 top-2 rounded-full bg-[#ff7e47] px-3 py-1 text-[10px] font-black text-white shadow-lg shadow-orange-300/40">加油呀</div>
              <Sparkles class="absolute bottom-5 right-4 h-5 w-5 text-amber-300" />
            </div>
          </div>

          <div class="absolute inset-0 pointer-events-none bg-[radial-gradient(circle_at_top_left,rgba(90,132,255,0.08),transparent_42%)]" />
        </div>

        <div class="home-hero p-5 text-white">
          <div class="grid grid-cols-[minmax(0,1fr)_110px] gap-4">
            <div class="min-w-0">
              <div class="flex items-center gap-2 text-sm font-bold text-blue-100">
                <CheckSquare class="h-4 w-4" />
                今日待复习
              </div>
              <div class="mt-4 flex items-end gap-2">
                <span class="text-6xl font-black leading-none">{{ store.todayCount }}</span>
                <span class="pb-2 text-sm font-semibold text-blue-100">张卡片</span>
              </div>
              <p class="mt-2 text-sm text-blue-100/90">预计耗时 {{ Math.max(1, Math.ceil(store.todayCount / 6)) }} 分钟</p>
            </div>
            <div class="flex items-center justify-center">
              <div class="relative h-[120px] w-[100px] rounded-[26px] bg-white/18 shadow-[inset_0_1px_0_rgba(255,255,255,0.24)]">
                <div class="absolute left-1/2 top-5 h-[82px] w-[62px] -translate-x-1/2 rounded-[18px] bg-white/95 shadow-[0_16px_32px_rgba(20,54,170,0.16)]" />
                <div class="absolute left-1/2 top-10 h-1 w-9 -translate-x-1/2 rounded-full bg-blue-300/70" />
                <div class="absolute left-1/2 top-[3.4rem] h-1 w-10 -translate-x-1/2 rounded-full bg-blue-200/75" />
                <div class="absolute left-1/2 top-[4.25rem] h-1 w-8 -translate-x-1/2 rounded-full bg-blue-200/65" />
                <div class="absolute bottom-5 right-4 h-10 w-10 rounded-2xl bg-white/28" />
              </div>
            </div>
          </div>
          <button
            class="primary-action mt-5 w-full text-sm"
            @click="goStudy()"
          >
            {{ store.todayCount === 0 ? '自由复习' : '开始学习' }}
            <ArrowRight class="h-4 w-4" />
          </button>
        </div>
      </section>

      <section class="mb-4">
        <div class="mb-3 flex items-center justify-between px-1">
          <div>
            <h2 class="text-base font-black text-ink">我的知识库</h2>
          </div>
          <button
            class="flex items-center gap-1 text-xs font-bold text-slate-400"
            @click="goLibrary"
          >
            查看全部
            <ArrowRight class="h-3.5 w-3.5" />
          </button>
        </div>

        <div v-if="store.decks.length > 0" class="soft-panel overflow-hidden px-4 py-3">
          <button
            v-for="deck in store.sortedDecks.slice(0, 3)"
            :key="deck.id"
            class="flex w-full items-center gap-3 border-b border-[#edf2ff] py-3 text-left last:border-b-0"
            @click="goDeckDetail(deck.id)"
          >
            <div class="deck-gem grid h-11 w-11 shrink-0 place-items-center rounded-2xl bg-gradient-to-br from-blue-500 to-cyan-400 text-white">
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

        <div v-else class="soft-panel p-7 text-center">
          <BookOpen class="mx-auto mb-3 h-12 w-12 text-blue-200" />
          <p class="mb-1 text-sm font-semibold text-slate-500">还没有知识库</p>
          <p class="text-xs muted">导入文本文件后就能开始学习</p>
        </div>
      </section>

      <section class="mb-4">
        <div class="mb-3 flex items-center justify-between px-1">
          <div>
            <h2 class="text-base font-black text-ink">学习方式</h2>
            <p class="mt-1 text-xs font-medium text-slate-400">按你喜欢的节奏开始</p>
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

      <section class="soft-panel p-4">
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
