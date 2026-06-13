<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft, ArrowRight, CheckCircle, Eye, Home, Loader, Pause, Play, RotateCcw, Settings, Volume2, VolumeX } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import { prepareSpeech, speakWord } from '../platform/tts.js'
import NavBar from '../components/NavBar.vue'
import BottomNav from '../components/BottomNav.vue'

const route = useRoute()
const router = useRouter()
const store = useAppStore()

const cards = ref([])
const currentIndex = ref(0)
const loading = ref(true)
const sessionDone = ref(false)
const answerVisible = ref(false)
const repeatCount = ref(2)
const intervalSeconds = ref(3)
const showPlaybackSettings = ref(false)
const isPlaying = ref(false)
const speechState = ref('idle')
const ttsUnavailable = ref(false)
const playToken = ref(0)
let speechController = null
let delayController = null

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

function wait(ms, signal) {
  return new Promise(resolve => {
    if (signal?.aborted) {
      resolve(false)
      return
    }
    const timer = setTimeout(() => {
      signal?.removeEventListener('abort', abort)
      resolve(true)
    }, ms)
    function abort() {
      clearTimeout(timer)
      resolve(false)
    }
    signal?.addEventListener('abort', abort, { once: true })
  })
}

function warmUpcoming(fromIndex, count = 3) {
  cards.value
    .slice(fromIndex, fromIndex + count)
    .forEach(card => prepareSpeech(card.word).catch(() => {}))
}

onMounted(async () => {
  try {
    cards.value = shuffle(await store.getTodayLearningCards(deckId.value))
    if (cards.value.length === 0) {
      sessionDone.value = true
    } else {
      warmUpcoming(0)
    }
  } catch (e) {
    console.warn('加载听写卡片失败:', e)
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

function togglePlaybackSettings() {
  showPlaybackSettings.value = !showPlaybackSettings.value
}

function closePlaybackSettings() {
  showPlaybackSettings.value = false
}

function stopPlayback() {
  playToken.value++
  speechController?.abort()
  speechController = null
  delayController?.abort()
  delayController = null
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
  isPlaying.value = true
  const token = ++playToken.value

  // 遍历所有剩余单词，自动连续播报
  while (token === playToken.value && currentIndex.value < cards.value.length) {
    const card = cards.value[currentIndex.value]
    const nextCard = cards.value[currentIndex.value + 1]
    if (nextCard) prepareSpeech(nextCard.word).catch(() => {})
    answerVisible.value = false
    ttsUnavailable.value = false

    for (let i = 0; i < repeatCount.value; i++) {
      if (token !== playToken.value) break
      const controller = new AbortController()
      speechController = controller
      try {
        await speakWord(card.word, {
          signal: controller.signal,
          timeoutMs: 30000,
          onStateChange: state => {
            if (token === playToken.value) speechState.value = state
          }
        })
      } catch {
        if (token === playToken.value) {
          ttsUnavailable.value = true
          speechState.value = 'unavailable'
        }
        break
      } finally {
        if (speechController === controller) speechController = null
      }

      if (i < repeatCount.value - 1 && token === playToken.value) {
        const controller = new AbortController()
        delayController = controller
        const completed = await wait(intervalSeconds.value * 1000, controller.signal)
        if (delayController === controller) delayController = null
        if (!completed) break
      }
    }

    // 当前词播完：如果不是最后一个词且未被中止，前进到下一个词
    if (token === playToken.value && currentIndex.value < cards.value.length - 1) {
      currentIndex.value++
      warmUpcoming(currentIndex.value + 1, 2)
      // 词间停顿
      const controller = new AbortController()
      delayController = controller
      const completed = await wait(intervalSeconds.value * 1000, controller.signal)
      if (delayController === controller) delayController = null
      if (!completed) break
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
  closePlaybackSettings()
  answerVisible.value = true
}

function goPrevious() {
  closePlaybackSettings()
  if (currentIndex.value === 0) return
  stopPlayback()
  currentIndex.value--
  answerVisible.value = false
  ttsUnavailable.value = false
}

function goNext() {
  closePlaybackSettings()
  if (isPlaying.value && currentIndex.value < cards.value.length - 1) {
    // Stop the old loop before changing the index so it cannot advance twice.
    stopPlayback()
    currentIndex.value++
    answerVisible.value = false
    ttsUnavailable.value = false
    startPlayback()
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
  closePlaybackSettings()
  currentIndex.value = 0
  answerVisible.value = false
  sessionDone.value = false
  ttsUnavailable.value = false
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
        <button
          v-if="!loading && !sessionDone"
          class="flex h-10 w-10 items-center justify-center rounded-full bg-white text-blue-500 shadow-sm"
          title="播报设置"
          @click="togglePlaybackSettings"
        >
          <Settings class="h-5 w-5" />
        </button>
      </template>
    </NavBar>

    <div v-if="!loading && !sessionDone && progress.total > 0" class="px-4 pt-2">
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

    <div v-else-if="currentCard" class="flex flex-1 flex-col px-4 py-5">
      <div class="flex flex-1 flex-col gap-5">
        <div v-if="showPlaybackSettings" class="soft-panel rounded-2xl p-4">
          <div class="mb-3 flex items-center justify-between">
            <div>
              <h2 class="text-sm font-black text-ink">播报设置</h2>
              <p class="mt-1 text-xs text-slate-400">调整重复次数和每次播报之间的停顿</p>
            </div>
            <button
              class="rounded-lg px-2 py-1 text-xs font-bold text-slate-400"
              @click="closePlaybackSettings"
            >
              关闭
            </button>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <label class="block">
              <span class="mb-1.5 block text-xs font-bold text-slate-500">重复次数</span>
              <select
                v-model="repeatCount"
                class="h-11 w-full rounded-xl border border-blue-100 bg-white px-3 text-sm text-ink outline-none focus:border-blue-300"
              >
                <option :value="1">1 次</option>
                <option :value="2">2 次</option>
                <option :value="3">3 次</option>
                <option :value="4">4 次</option>
                <option :value="5">5 次</option>
              </select>
            </label>

            <label class="block">
              <span class="mb-1.5 block text-xs font-bold text-slate-500">播报间隔</span>
              <select
                v-model="intervalSeconds"
                class="h-11 w-full rounded-xl border border-blue-100 bg-white px-3 text-sm text-ink outline-none focus:border-blue-300"
              >
                <option :value="1">1 秒</option>
                <option :value="2">2 秒</option>
                <option :value="3">3 秒</option>
                <option :value="4">4 秒</option>
                <option :value="5">5 秒</option>
                <option :value="6">6 秒</option>
                <option :value="8">8 秒</option>
                <option :value="10">10 秒</option>
              </select>
            </label>
          </div>
        </div>

        <div class="soft-panel flex-1 rounded-[32px] px-6 py-8 text-center">
            <div
              class="mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full"
              :class="isPlaying ? 'bg-blue-50 text-blue-600' : 'bg-slate-100 text-slate-400'"
            >
              <Loader v-if="speechState === 'loading'" class="h-8 w-8 animate-spin" />
              <VolumeX v-else-if="speechState === 'unavailable'" class="h-8 w-8" />
              <Volume2 v-else class="h-8 w-8" />
            </div>

            <h1 class="mb-1 text-[2rem] font-black tracking-[-0.03em] text-ink">{{ answerVisible ? currentCard.word : '听发音，写下单词' }}</h1>
            <p v-if="answerVisible && currentCard.inflections?.length" class="mb-5 text-sm font-bold text-blue-500">
              {{ currentCard.inflections.join(' · ') }}
            </p>

            <div class="mx-auto my-8 flex h-12 max-w-[260px] items-center justify-center gap-1 text-blue-400">
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

            <p class="mt-4 text-xs font-medium text-slate-400">请在纸上完成听写，需要时再查看答案</p>

            <p v-if="answerVisible && currentCard.definition" class="mt-4 text-sm leading-relaxed text-slate-600">
              {{ currentCard.definition }}
            </p>
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
