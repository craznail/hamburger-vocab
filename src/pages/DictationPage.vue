<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, ArrowRight, CheckCircle, Eye, Home, Loader, Pause, Play, RotateCcw, Settings, Volume2, VolumeX } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import { speakWord } from '../utils/speech.js'
import NavBar from '../components/NavBar.vue'

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
  <div class="min-h-screen flex flex-col bg-[#fafafa]">
    <NavBar @back="goHome">
      <template #right>
        <div v-if="!loading && !sessionDone" class="text-sm text-gray-400">
          {{ progress.current }} / {{ progress.total }}
        </div>
      </template>
    </NavBar>

    <div v-if="loading" class="flex-1 flex items-center justify-center">
      <p class="text-gray-400">加载中...</p>
    </div>

    <div v-else-if="sessionDone" class="flex-1 flex items-center justify-center px-5">
      <div class="w-full max-w-md text-center">
        <CheckCircle class="w-16 h-16 mx-auto mb-4 text-green-400" />
        <h2 class="text-xl font-semibold text-gray-700 mb-2">
          {{ cards.length === 0 ? '今日无待听写' : '本轮播报完成' }}
        </h2>
        <p class="text-gray-400 mb-6">
          {{ cards.length === 0 ? '当前没有到期单词，明天再来吧' : `共播报 ${cards.length} 个单词` }}
        </p>
        <div class="flex justify-center gap-3">
          <button
            v-if="cards.length > 0"
            class="inline-flex items-center gap-2 px-5 py-2.5 bg-blue-600 text-white rounded-xl hover:bg-blue-700 transition-colors cursor-pointer"
            @click="restartSession"
          >
            <RotateCcw class="w-4 h-4" />
            再来一轮
          </button>
          <button
            class="inline-flex items-center gap-2 px-5 py-2.5 border border-gray-300 text-gray-600 rounded-xl hover:bg-gray-50 transition-colors cursor-pointer"
            @click="goHome"
          >
            <Home class="w-4 h-4" />
            返回首页
          </button>
        </div>
      </div>
    </div>

    <div v-else-if="currentCard" class="flex-1 flex flex-col px-5 py-5">
      <div class="w-full max-w-2xl mx-auto flex-1 flex flex-col gap-5">
        <div class="bg-white border border-gray-100 shadow-sm rounded-xl p-4">
          <div class="flex items-center gap-2 text-gray-600 mb-4">
            <Settings class="w-4 h-4" />
            <span class="text-sm font-medium">播报设置</span>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <label class="text-sm text-gray-500">
              播报次数
              <input
                v-model.number="repeatCount"
                type="number"
                min="1"
                max="5"
                class="mt-1 w-full px-3 py-2 border border-gray-200 rounded-lg text-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-200 focus:border-blue-400"
                @blur="clampSettings"
              />
            </label>
            <label class="text-sm text-gray-500">
              间隔秒数
              <input
                v-model.number="intervalSeconds"
                type="number"
                min="1"
                max="10"
                class="mt-1 w-full px-3 py-2 border border-gray-200 rounded-lg text-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-200 focus:border-blue-400"
                @blur="clampSettings"
              />
            </label>
          </div>
        </div>

        <div class="flex-1 bg-white shadow-lg border border-gray-100 rounded-xl overflow-hidden flex flex-col">
          <div class="w-full h-1.5 bg-gray-200 overflow-hidden">
            <div
              class="h-full bg-blue-500 transition-all duration-300"
              :style="{ width: `${Math.round((progress.current / progress.total) * 100)}%` }"
            />
          </div>

          <div class="flex-1 flex flex-col items-center justify-center px-5 py-8 text-center">
            <div
              class="w-20 h-20 rounded-full flex items-center justify-center mb-5"
              :class="isPlaying ? 'bg-blue-50 text-blue-600' : 'bg-gray-100 text-gray-400'"
            >
              <Loader v-if="speechState === 'loading'" class="w-9 h-9 animate-spin" />
              <VolumeX v-else-if="speechState === 'unavailable'" class="w-9 h-9" />
              <Volume2 v-else class="w-9 h-9" />
            </div>

            <h1 class="text-xl sm:text-2xl font-semibold text-gray-700 mb-2">请听发音并在纸上写下单词</h1>
            <p class="text-sm text-gray-400 mb-6">
              当前单词会播报 {{ repeatCount }} 次，每次间隔 {{ intervalSeconds }} 秒
            </p>

            <p v-if="ttsUnavailable" class="text-xs text-gray-400 mb-4">发音不可用，请检查网络或 TTS 设置</p>

            <div class="flex flex-wrap justify-center gap-3">
              <button
                class="inline-flex items-center gap-2 px-6 py-3 bg-blue-600 text-white rounded-xl hover:bg-blue-700 transition-colors cursor-pointer"
                @click="togglePlayback"
              >
                <Pause v-if="isPlaying" class="w-4 h-4" />
                <Play v-else class="w-4 h-4" />
                {{ isPlaying ? '暂停' : '开始播报' }}
              </button>
              <button
                class="inline-flex items-center gap-2 px-6 py-3 border border-gray-300 text-gray-600 rounded-xl hover:bg-gray-50 transition-colors cursor-pointer"
                @click="revealAnswer"
              >
                <Eye class="w-4 h-4" />
                看答案
              </button>
            </div>
          </div>

          <div v-if="answerVisible" class="border-t border-gray-100 px-5 py-6 text-center">
            <h2 class="text-4xl sm:text-5xl font-bold text-gray-800 mb-3">{{ currentCard.word }}</h2>
            <div
              v-if="currentCard.inflections && currentCard.inflections.length"
              class="text-lg sm:text-xl text-gray-500 mb-3"
            >
              {{ currentCard.inflections.join(' · ') }}
            </div>
            <p v-if="currentCard.definition" class="text-base sm:text-lg text-gray-600 leading-relaxed">
              {{ currentCard.definition }}
            </p>
            <p v-else class="text-base sm:text-lg text-gray-400 italic">暂无释义</p>
          </div>
        </div>

        <div class="flex justify-between gap-3">
          <button
            class="inline-flex items-center justify-center gap-2 flex-1 px-4 py-3 border border-gray-300 text-gray-600 rounded-xl hover:bg-gray-50 disabled:opacity-40 disabled:cursor-not-allowed transition-colors cursor-pointer"
            :disabled="currentIndex === 0"
            @click="goPrevious"
          >
            <ArrowLeft class="w-4 h-4" />
            上一词
          </button>
          <button
            class="inline-flex items-center justify-center gap-2 flex-1 px-4 py-3 bg-blue-600 text-white rounded-xl hover:bg-blue-700 transition-colors cursor-pointer"
            @click="goNext"
          >
            {{ currentIndex === cards.length - 1 ? '完成' : '下一词' }}
            <ArrowRight class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
