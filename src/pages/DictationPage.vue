<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, ArrowRight, CheckCircle, Eye, Heart, Home, Loader, Pause, Play, RotateCcw, Settings, Star, Trees, Volume2, VolumeX } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import { speakWord } from '../platform/tts.js'
import NavBar from '../components/NavBar.vue'
import BottomNav from '../components/BottomNav.vue'

const route = useRoute()
const router = useRouter()
const store = useAppStore()

const cards = ref([])
const currentIndex = ref(0)
const loading = ref(true)
const sessionDone = ref(false)
const skipping = ref(false)
const answerVisible = ref(false)
const repeatCount = ref(2)
const intervalSeconds = ref(3)
const isPlaying = ref(false)
const speechState = ref('idle')
const ttsUnavailable = ref(false)
const playToken = ref(0)
let speechController = null

const deckId = computed(() => route.query.deckId || null)
const currentCard = computed(() => cards.value[currentIndex.value] || null)
const progress = computed(() => ({
  current: cards.value.length === 0 ? 0 : currentIndex.value + 1,
  total: cards.value.length
}))

function shuffle(array) {
  const a = [...array]
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]]
  }
  return a
}

function clampSettings() {
  repeatCount.value = Math.min(5, Math.max(1, Number(repeatCount.value) || 2))
  intervalSeconds.value = Math.min(10, Math.max(1, Number(intervalSeconds.value) || 3))
}

function wait(ms) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

onMounted(async () => {
  try {
    cards.value = shuffle(await store.getTodayLearningCards(deckId.value))
    if (cards.value.length === 0) {
      sessionDone.value = true
    }
  } catch (e) {
    console.warn('加载听写卡片失败（预览模式时正常）:', e)
    cards.value = []
    sessionDone.value = true
  } finally {
    loading.value = false
  }
})

onBeforeUnmount(() => {
  stopPlayback()
})

function goHome() {
  router.push({ name: 'Home' })
}

function stopPlayback() {
  playToken.value++
  speechController?.abort()
  speechController = null
  isPlaying.value = false
  speechState.value = 'idle'
  if ('speechSynthesis' in window) {
    speechSynthesis.cancel()
  }
}

async function startPlayback() {
  if (!currentCard.value || isPlaying.value) return
  clampSettings()
  ttsUnavailable.value = false
  skipping.value = false
  isPlaying.value = true
  const token = ++playToken.value

  // 遍历所有剩余单词，自动连续播报
  while (token === playToken.value && currentIndex.value < cards.value.length) {
    const card = cards.value[currentIndex.value]
    answerVisible.value = false
    ttsUnavailable.value = false

    for (let i = 0; i < repeatCount.value; i++) {
      if (token !== playToken.value) break
      speechController = new AbortController()
      try {
        await speakWord(card.word, {
          signal: speechController.signal,
          timeoutMs: 30000,
          onStateChange: state => {
            if (token === playToken.value) speechState.value = state
          }
        })
      } catch {
        if (token === playToken.value) {
          if (skipping.value) {
            // 用户主动跳过当前词，重置标记继续下一个词
            skipping.value = false
          } else {
            ttsUnavailable.value = true
            speechState.value = 'unavailable'
          }
        }
        break
      } finally {
        speechController = null
      }

      if (i < repeatCount.value - 1 && token === playToken.value) {
        await wait(intervalSeconds.value * 1000)
      }
    }

    // 当前词播完：如果不是最后一个词且未被中止，前进到下一个词
    if (token === playToken.value && currentIndex.value < cards.value.length - 1) {
      currentIndex.value++
      // 词间停顿
      await wait(intervalSeconds.value * 1000)
    } else {
      break
    }
  }

  if (token === playToken.value) {
    if (currentIndex.value >= cards.value.length - 1 && cards.value.length > 0) {
      sessionDone.value = true
    }
    isPlaying.value = false
    speechState.value = ttsUnavailable.value ? 'unavailable' : 'idle'
  }
}

function togglePlayback() {
  if (isPlaying.value) {
    stopPlayback()
  } else {
    startPlayback()
  }
}

function revealAnswer() {
  answerVisible.value = true
}

function goPrevious() {
  if (currentIndex.value === 0) return
  stopPlayback()
  currentIndex.value--
  answerVisible.value = false
  ttsUnavailable.value = false
}

function goNext() {
  if (isPlaying.value && currentIndex.value < cards.value.length - 1) {
    // 播放中切下一词：跳过当前词继续播
    skipping.value = true
    speechController?.abort()
    speechController = null
    currentIndex.value++
    answerVisible.value = false
    ttsUnavailable.value = false
  } else {
    // 不在播放中：原行为
    stopPlayback()
    if (currentIndex.value < cards.value.length - 1) {
      currentIndex.value++
      answerVisible.value = false
      ttsUnavailable.value = false
    } else {
      sessionDone.value = true
    }
  }
}

function restartSession() {
  stopPlayback()
  currentIndex.value = 0
  answerVisible.value = false
  sessionDone.value = false
  ttsUnavailable.value = false
  skipping.value = false
}

</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar @back="goHome">
      <template #left>
        <div v-if="!loading && !sessionDone">
          <h1 class="text-sm font-black text-ink">听写模式</h1>
          <p class="mt-1 text-xs text-slate-400">{{ progress.current }} / {{ progress.total }}</p>
        </div>
      </template>
      <template #right>
        <button v-if="!loading && !sessionDone" class="flex h-10 w-10 items-center justify-center rounded-full bg-white text-blue-500 shadow-sm" title="播报设置">
          <Settings class="h-5 w-5" />
        </button>
      </template>
    </NavBar>

    <div v-if="!loading && !sessionDone && progress.total > 0" class="px-5">
      <div class="progress-track h-1.5">
        <div class="progress-fill" :style="{ width: `${Math.round((progress.current / progress.total) * 100)}%` }" />
      </div>
    </div>

    <div v-if="loading" class="flex flex-1 items-center justify-center">
      <p class="text-slate-400">加载中...</p>
    </div>

    <div v-else-if="sessionDone" class="flex flex-1 items-center justify-center px-5">
      <div class="soft-panel w-full rounded-2xl p-8 text-center">
        <CheckCircle class="mx-auto mb-4 h-16 w-16 text-green-400" />
        <h2 class="mb-2 text-xl font-black text-ink">
          {{ cards.length === 0 ? '今日无待听写' : '本轮播报完成' }}
        </h2>
        <p class="mb-6 text-sm text-slate-400">
          {{ cards.length === 0 ? '当前没有到期单词，明天再来吧' : `共播报 ${cards.length} 个单词` }}
        </p>
        <div class="flex justify-center gap-3">
          <button
            v-if="cards.length > 0"
            class="blue-gradient inline-flex h-11 items-center gap-2 rounded-xl px-5 text-sm font-bold text-white"
            @click="restartSession"
          >
            <RotateCcw class="h-4 w-4" />
            再来一轮
          </button>
          <button
            class="inline-flex h-11 items-center gap-2 rounded-xl border border-blue-100 bg-white px-5 text-sm font-bold text-slate-500"
            @click="goHome"
          >
            <Home class="h-4 w-4" />
            返回首页
          </button>
        </div>
      </div>
    </div>

    <div v-else-if="currentCard" class="flex flex-1 flex-col px-5 py-5">
      <div class="flex flex-1 flex-col gap-5">
        <div class="soft-panel flex-1 rounded-[24px] px-5 py-8 text-center">
            <div
              class="mx-auto mb-5 flex h-16 w-16 items-center justify-center rounded-full"
              :class="isPlaying ? 'bg-blue-50 text-blue-600' : 'bg-slate-100 text-slate-400'"
            >
              <Loader v-if="speechState === 'loading'" class="h-8 w-8 animate-spin" />
              <VolumeX v-else-if="speechState === 'unavailable'" class="h-8 w-8" />
              <Volume2 v-else class="h-8 w-8" />
            </div>

            <h1 class="mb-1 text-2xl font-black text-ink">{{ answerVisible ? currentCard.word : '听发音，写下单词' }}</h1>
            <p v-if="answerVisible && currentCard.inflections?.length" class="mb-5 text-sm font-bold text-blue-500">
              {{ currentCard.inflections.join(' · ') }}
            </p>

            <div class="mx-auto my-6 flex h-12 max-w-[260px] items-center justify-center gap-1 text-blue-400">
              <span v-for="n in 28" :key="n" class="w-1 rounded-full bg-current" :style="{ height: `${12 + ((n * 7) % 28)}px`, opacity: 0.35 + ((n % 5) * 0.12) }" />
            </div>

            <p v-if="ttsUnavailable" class="mb-4 text-xs text-slate-400">发音不可用，请检查网络或 TTS 设置</p>

            <div class="mb-5 flex flex-wrap justify-center gap-3">
              <button
                class="blue-gradient inline-flex h-11 items-center gap-2 rounded-xl px-6 text-sm font-bold text-white"
                @click="togglePlayback"
              >
                <Pause v-if="isPlaying" class="h-4 w-4" />
                <Play v-else class="h-4 w-4" />
                {{ isPlaying ? '暂停' : '开始播报' }}
              </button>
              <button
                class="inline-flex h-11 items-center gap-2 rounded-xl border border-blue-100 bg-white px-6 text-sm font-bold text-slate-500"
                @click="revealAnswer"
              >
                <Eye class="h-4 w-4" />
                看答案
              </button>
            </div>

            <input class="h-14 w-full rounded-xl border border-blue-100 bg-white px-4 text-center text-sm text-ink outline-none focus:border-blue-300" placeholder="请输入你听到的单词" />
            <p class="mt-4 text-xs text-slate-400">不知道？点击查看答案</p>

            <p v-if="answerVisible && currentCard.definition" class="mt-4 text-sm leading-relaxed text-slate-600">
              {{ currentCard.definition }}
            </p>
        </div>

        <div class="grid grid-cols-3 gap-4">
          <button class="red-gradient flex min-h-[74px] flex-col items-center justify-center gap-1 rounded-2xl text-white shadow-lg shadow-red-200/60" @click="goNext">
            <Heart class="h-6 w-6" />
            <span class="text-sm font-black">忘记</span>
            <span class="text-[10px] text-white/75">1 天后复习</span>
          </button>
          <button class="warm-gradient flex min-h-[74px] flex-col items-center justify-center gap-1 rounded-2xl text-white shadow-lg shadow-amber-200/60" @click="goNext">
            <Star class="h-6 w-6" />
            <span class="text-sm font-black">模糊</span>
            <span class="text-[10px] text-white/75">3 天后复习</span>
          </button>
          <button class="green-gradient flex min-h-[74px] flex-col items-center justify-center gap-1 rounded-2xl text-white shadow-lg shadow-green-200/60" @click="goNext">
            <Trees class="h-6 w-6" />
            <span class="text-sm font-black">认识</span>
            <span class="text-[10px] text-white/75">7 天后复习</span>
          </button>
        </div>

        <div class="flex justify-between gap-3">
          <button
            class="inline-flex h-11 flex-1 items-center justify-center gap-2 rounded-xl border border-blue-100 bg-white px-4 text-sm font-bold text-slate-500 disabled:opacity-40"
            :disabled="currentIndex === 0"
            @click="goPrevious"
          >
            <ArrowLeft class="h-4 w-4" />
            上一词
          </button>
          <button
            class="blue-gradient inline-flex h-11 flex-1 items-center justify-center gap-2 rounded-xl px-4 text-sm font-bold text-white"
            @click="goNext"
          >
            {{ currentIndex === cards.length - 1 ? '完成' : '下一词' }}
            <ArrowRight class="h-4 w-4" />
          </button>
        </div>
      </div>
    </div>
    <BottomNav />
  </div>
</template>
