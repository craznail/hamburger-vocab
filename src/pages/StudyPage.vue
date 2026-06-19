<script setup>
import { ref, computed, onMounted, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore } from '../stores/useAppStore'
import { BarChart3, CheckCircle } from 'lucide-vue-next'
import FlashCard from '../components/FlashCard.vue'
import BottomNav from '../components/BottomNav.vue'
import { useErrorNotebookStore } from '../stores/useErrorNotebookStore'

const route = useRoute()
const router = useRouter()
const store = useAppStore()
const errorStore = useErrorNotebookStore()
const studyPlanIcon = new URL('../assets/icons/study-plan-icon.svg', import.meta.url).href
const studyDictationIcon = new URL('../assets/icons/study-dictation-tile.svg', import.meta.url).href
const studyErrorIcon = new URL('../assets/icons/study-error-tile.svg', import.meta.url).href

const cards = ref([])
const currentIndex = ref(0)
const sessionResults = ref([])
const sessionDone = ref(false)
const cardStartedAt = ref(Date.now())
const mode = ref('review')
const paused = ref(false)
const activeDeck = ref(null)
const usingDevMockData = ref(false)

const deckId = computed(() => route.query.deckId || null)
const forcePractice = computed(() => route.query.mode === 'practice')
const isPreviewMode = computed(() => import.meta.env.DEV && route.query.preview === '1')
const allowDevMockFallback = computed(() => import.meta.env.DEV && route.query.real !== '1')
const currentCard = computed(() => cards.value[currentIndex.value] || null)
const isPractice = computed(() => mode.value === 'practice')
const pageTitle = computed(() => route.query.deckName || (isPractice.value ? '自由练习' : '今日学习'))
const progress = computed(() => ({
  current: cards.value.length === 0 ? 0 : currentIndex.value + 1,
  total: cards.value.length
}))
const estimatedMinutes = computed(() => Math.max(1, Math.ceil(progress.value.total / 3)))
const dictationCount = computed(() => {
  const fromDeck = Number(activeDeck.value?.wordCount || activeDeck.value?.word_count || activeDeck.value?.total || 0)
  return Math.max(1, Math.min(12, fromDeck || cards.value.length || 12))
})
const errorReviewCount = computed(() => Math.max(1, errorStore.dueCount || forgotCount.value || 3))

const masteredCount = computed(() =>
  sessionResults.value.filter(r => r.quality === 5).length
)
const hazyCount = computed(() =>
  sessionResults.value.filter(r => r.quality === 3).length
)
const forgotCount = computed(() =>
  sessionResults.value.filter(r => r.quality === 0).length
)

function buildPreviewCards() {
  return Array.from({ length: 18 }, (_, index) => ({
    id: `preview-${index + 1}`,
    word: 'abandon',
    inflections: ['əˈbændən'],
    definition: '放弃；抛弃',
    ef: 2.5,
    interval: 6n,
    repetitions: 2n,
    nextReview: '2026-06-20',
    createdAt: '2026-06-01',
    lastReviewAt: '2026-06-18',
    deckName: '高考核心词',
  }))
}

function loadPreviewSession() {
  usingDevMockData.value = true
  cards.value = buildPreviewCards()
  currentIndex.value = 7
  sessionResults.value = []
  sessionDone.value = false
  paused.value = false
  mode.value = 'review'
  activeDeck.value = {
    id: 'preview-deck',
    name: '高考核心词',
    wordCount: 1280,
  }
  cardStartedAt.value = Date.now()
}

function shuffle(array) {
  const a = [...array]
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]]
  }
  return a
}

onMounted(async () => {
  if (isPreviewMode.value) {
    loadPreviewSession()
    return
  }
  void errorStore.ensureFresh()
  if (deckId.value) {
    activeDeck.value = await store.getDeckInfo(deckId.value)
  }
  await loadCards({ forcePractice: forcePractice.value })
})

async function loadCards({ forcePractice = false } = {}) {
  try {
    usingDevMockData.value = false
    let loadedCards = []
    if (!forcePractice) {
      loadedCards = await store.getTodayLearningCards(deckId.value)
    }
    if (loadedCards.length > 0) {
      mode.value = 'review'
    } else {
      mode.value = 'practice'
      loadedCards = await store.getPracticeCards(deckId.value)
    }
    cards.value = shuffle(loadedCards)
    cardStartedAt.value = Date.now()
    paused.value = false
  } catch (e) {
    console.warn('加载学习卡片失败:', e)
    cards.value = []
  }
  if (cards.value.length === 0) {
    if (allowDevMockFallback.value) {
      loadPreviewSession()
      return
    }
    sessionDone.value = true
  }
}

async function handleRate(quality) {
  if (!currentCard.value || paused.value) return

  const card = currentCard.value

  if (usingDevMockData.value) {
    sessionResults.value.push({
      word: card.word,
      quality,
      definition: card.definition
    })

    if (currentIndex.value < cards.value.length - 1) {
      currentIndex.value++
      cardStartedAt.value = Date.now()
    } else {
      sessionDone.value = true
    }
    return
  }

  const durationSeconds = Math.max(1, Math.round((Date.now() - cardStartedAt.value) / 1000))
  if (isPractice.value) {
    await store.ratePracticeCard(card.id, quality, durationSeconds)
  } else {
    await store.rateCard(card.id, quality, durationSeconds)
  }

  sessionResults.value.push({
    word: card.word,
    quality,
    definition: card.definition
  })

  if (currentIndex.value < cards.value.length - 1) {
    currentIndex.value++
    cardStartedAt.value = Date.now()
  } else {
    sessionDone.value = true
    // Refresh global stats once when the session completes
    store.refreshAll()
  }
}

function goHome() {
  router.push({ name: 'Home' })
}

function goPlan() {
  router.push({ name: 'Stats' })
}

function goDictation() {
  router.push({
    name: 'Dictation',
    query: deckId.value ? { deckId: deckId.value } : {},
  })
}

function goErrorReview() {
  router.push({ name: 'ErrorReview' })
}

function togglePause() {
  paused.value = !paused.value
}

async function continueSession() {
  paused.value = false
  await nextTick()
  const active = document.querySelector('.flashcard-panel')
  if (active instanceof HTMLElement) {
    active.scrollIntoView({ behavior: 'smooth', block: 'center' })
  }
}

async function continueStudy() {
  if (usingDevMockData.value) {
    loadPreviewSession()
    return
  }
  sessionDone.value = false
  currentIndex.value = 0
  sessionResults.value = []
  await loadCards({ forcePractice: isPractice.value })
}
</script>

<template>
  <div class="study-page app-page flex min-h-screen flex-col">
    <div class="study-shell">
      <header class="study-topbar">
        <div class="min-w-0">
          <h1 class="study-title">{{ pageTitle }}</h1>
          <p class="study-subtitle">先完成最小一组，保持节奏</p>
        </div>
        <button v-if="!sessionDone" class="study-plan-action" title="复习计划" @click="goPlan">
          <img class="study-plan-action-icon" :src="studyPlanIcon" alt="" aria-hidden="true" />
          <span>复习计划</span>
        </button>
      </header>

      <div class="flex flex-1 flex-col">
      <div v-if="sessionDone" class="flex flex-1 flex-col justify-start text-center">
        <div v-if="cards.length === 0" class="soft-panel relative mt-4 w-full overflow-hidden p-8">
          <div class="absolute right-[-2rem] top-[-2rem] h-28 w-28 rounded-[2rem] bg-blue-50 rotate-12" />
          <CheckCircle class="relative mx-auto mb-4 h-16 w-16 text-[#2fcfa6]" />
          <h2 class="relative mb-2 text-xl font-black text-ink">暂无可练习卡片</h2>
          <p class="relative mb-6 text-sm text-slate-400">当前范围内还没有卡片，可以先导入知识库或回到首页自由浏览。</p>
          <button
            class="blue-gradient h-11 rounded-xl px-6 text-sm font-bold text-white"
            @click="goHome"
          >
            返回首页
          </button>
        </div>
        <div v-else class="soft-panel w-full p-8">
          <BarChart3 class="mx-auto mb-4 h-16 w-16 text-blue-400" />
          <h2 class="mb-1 text-xl font-black text-ink">{{ isPractice ? '自由练习完成' : '今日复习完成' }}</h2>
          <p class="mb-1 text-sm text-slate-400">共练习 {{ cards.length }} 张卡片</p>
          <p v-if="isPractice" class="mb-6 text-xs text-slate-400">本轮结果仅记录练习，不影响复习计划</p>
          <div v-else class="mb-5" />

          <div class="mb-6 grid grid-cols-3 gap-3">
            <div class="text-center">
              <div class="text-2xl font-black text-green-500">{{ masteredCount }}</div>
              <div class="text-xs text-slate-400">已掌握</div>
            </div>
            <div class="text-center">
              <div class="text-2xl font-black text-amber-400">{{ hazyCount }}</div>
              <div class="text-xs text-slate-400">模糊</div>
            </div>
            <div class="text-center">
              <div class="text-2xl font-black text-red-500">{{ forgotCount }}</div>
              <div class="text-xs text-slate-400">忘记</div>
            </div>
          </div>

          <div class="flex justify-center gap-3">
            <button
              class="blue-gradient h-11 rounded-xl px-6 text-sm font-bold text-white"
              @click="continueStudy"
            >
              {{ isPractice ? '再练一轮' : '继续学习' }}
            </button>
            <button
              class="h-11 rounded-xl border border-blue-100 bg-white px-6 text-sm font-bold text-slate-500"
              @click="goHome"
            >
              返回首页
            </button>
          </div>
        </div>
      </div>

      <template v-else-if="currentCard">
        <FlashCard
          :key="currentCard.id"
          :card="currentCard"
          :current="progress.current"
          :total="cards.length"
          :practice-mode="isPractice"
          :paused="paused"
          :estimated-minutes="estimatedMinutes"
          :deck-name="route.query.deckName || currentCard.deckName"
          class="flex-1"
          @rate="handleRate"
          @toggle-pause="togglePause"
          @continue-study="continueSession"
        />

        <div class="study-module-list">
          <button class="study-module-card" type="button" @click="goDictation">
            <img class="study-module-icon" :src="studyDictationIcon" alt="" aria-hidden="true" />
            <span class="study-module-copy">
              <span class="study-module-title">听写训练 {{ dictationCount }} 词</span>
              <span class="study-module-desc">强化拼写，巩固记忆</span>
            </span>
            <span class="study-module-chevron">›</span>
          </button>

          <button class="study-module-card" type="button" @click="goErrorReview">
            <img class="study-module-icon" :src="studyErrorIcon" alt="" aria-hidden="true" />
            <span class="study-module-copy">
              <span class="study-module-title">错题复习 {{ errorReviewCount }} 道</span>
              <span class="study-module-desc">针对错题，查漏补缺</span>
            </span>
            <span class="study-module-chevron">›</span>
          </button>
        </div>
      </template>
    </div>
    </div>
    <BottomNav />
  </div>
</template>

<style scoped>
.study-page {
  background:
    linear-gradient(rgba(89, 122, 194, 0.015) 1px, transparent 1px),
    linear-gradient(90deg, rgba(89, 122, 194, 0.015) 1px, transparent 1px),
    radial-gradient(circle at 24% 0%, rgba(110, 154, 245, 0.075), transparent 32%),
    linear-gradient(180deg, #fffefe 0%, #f7faff 42%, #eef4fc 100%);
  background-size: 28px 28px, 28px 28px, auto, auto;
}

.study-shell {
  flex: 1;
  padding: calc(var(--safe-area-top) + 0.96rem) 1rem 6.95rem;
}

.study-topbar {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.9rem;
  margin-bottom: 0.94rem;
  padding: 0.15rem 0.1rem 0;
}

.study-title {
  margin: 0;
  color: #1a326c;
  font-size: 1.82rem;
  line-height: 1;
  font-weight: 950;
}

.study-subtitle {
  margin: 0.34rem 0 0;
  color: #657ba9;
  font-size: 0.84rem;
  line-height: 1.38;
  font-weight: 650;
}

.study-plan-action {
  display: inline-flex;
  align-items: center;
  gap: 0.52rem;
  min-height: 2.18rem;
  border: 1px solid rgba(214, 223, 241, 0.95);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.94);
  padding: 0 0.84rem;
  color: #214fd8;
  font-size: 0.82rem;
  font-weight: 800;
  box-shadow:
    0 8px 18px rgba(80, 104, 156, 0.04),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.study-plan-action-icon {
  width: 1.28rem;
  height: 1.28rem;
  flex: 0 0 auto;
}

.study-module-list {
  display: grid;
  gap: 0.5rem;
  margin-top: 0.94rem;
}

.study-module-card {
  display: flex;
  align-items: center;
  gap: 0.94rem;
  width: 100%;
  border: 1px solid rgba(219, 228, 244, 0.94);
  border-radius: 1.18rem;
  background:
    radial-gradient(circle at 78% 12%, rgba(93, 133, 225, 0.06), transparent 24%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.97) 0%, rgba(251, 253, 255, 0.94) 100%);
  padding: 0.94rem 0.98rem;
  text-align: left;
  box-shadow:
    0 10px 22px rgba(75, 104, 164, 0.034),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.study-module-icon {
  width: 3.28rem;
  height: 3.28rem;
  flex: 0 0 auto;
}

.study-module-copy {
  display: grid;
  min-width: 0;
  flex: 1;
}

.study-module-title {
  color: #1c336e;
  font-size: 0.98rem;
  font-weight: 900;
}

.study-module-desc {
  margin-top: 0.24rem;
  color: #7f91b8;
  font-size: 0.8rem;
  font-weight: 650;
}

.study-module-chevron {
  color: #91a2c3;
  font-size: 1.6rem;
  line-height: 1;
}
</style>
