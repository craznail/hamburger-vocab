<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore } from '../stores/useAppStore'
import { BarChart3, CheckCircle } from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
import FlashCard from '../components/FlashCard.vue'
import BottomNav from '../components/BottomNav.vue'

const route = useRoute()
const router = useRouter()
const store = useAppStore()

const cards = ref([])
const currentIndex = ref(0)
const sessionResults = ref([])
const sessionDone = ref(false)

const deckId = computed(() => route.query.deckId || null)
const currentCard = computed(() => cards.value[currentIndex.value] || null)
const progress = computed(() => ({
  current: cards.value.length === 0 ? 0 : currentIndex.value + 1,
  total: cards.value.length
}))

const masteredCount = computed(() =>
  sessionResults.value.filter(r => r.quality === 5).length
)
const hazyCount = computed(() =>
  sessionResults.value.filter(r => r.quality === 3).length
)
const forgotCount = computed(() =>
  sessionResults.value.filter(r => r.quality === 0).length
)

function shuffle(array) {
  const a = [...array]
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]]
  }
  return a
}

onMounted(async () => {
  await loadCards()
})

async function loadCards() {
  try {
    cards.value = shuffle(await store.getTodayLearningCards(deckId.value))
  } catch (e) {
    console.warn('加载学习卡片失败（预览模式时正常）:', e)
    cards.value = []
  }
  if (cards.value.length === 0) {
    sessionDone.value = true
  }
}

async function handleRate(quality) {
  if (!currentCard.value) return

  const card = currentCard.value
  await store.rateCard(card.id, quality)

  sessionResults.value.push({
    word: card.word,
    quality,
    definition: card.definition
  })

  if (currentIndex.value < cards.value.length - 1) {
    currentIndex.value++
  } else {
    sessionDone.value = true
  }
}

function goHome() {
  router.push({ name: 'Home' })
}

async function continueStudy() {
  sessionDone.value = false
  currentIndex.value = 0
  sessionResults.value = []
  await loadCards()
}
</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar @back="goHome">
      <template #left>
        <div v-if="!sessionDone">
          <h1 class="text-sm font-black text-ink">闪卡学习</h1>
          <p class="mt-1 text-xs text-slate-400">{{ progress.current }} / {{ progress.total }}</p>
        </div>
      </template>
    </NavBar>

    <div v-if="!sessionDone && progress.total > 0" class="px-5">
      <div class="progress-track h-1.5">
        <div class="progress-fill" :style="{ width: `${Math.round((progress.current / progress.total) * 100)}%` }" />
      </div>
    </div>

    <div class="flex flex-1 flex-col px-5 pt-5">
      <div v-if="sessionDone" class="flex flex-1 items-center justify-center text-center">
        <div v-if="cards.length === 0" class="soft-panel w-full rounded-2xl p-8">
          <CheckCircle class="mx-auto mb-4 h-16 w-16 text-green-400" />
          <h2 class="mb-2 text-xl font-black text-ink">今日无待复习</h2>
          <p class="mb-6 text-sm text-slate-400">所有卡片已复习完毕，明天再来吧</p>
          <button
            class="blue-gradient h-11 rounded-xl px-6 text-sm font-bold text-white"
            @click="goHome"
          >
            返回首页
          </button>
        </div>
        <div v-else class="soft-panel w-full rounded-2xl p-8">
          <BarChart3 class="mx-auto mb-4 h-16 w-16 text-blue-400" />
          <h2 class="mb-1 text-xl font-black text-ink">今日学习完成</h2>
          <p class="mb-6 text-sm text-slate-400">共复习 {{ cards.length }} 张卡片</p>

          <div class="mb-6 grid grid-cols-3 gap-3">
            <div class="text-center">
              <div class="text-2xl font-black text-green-500">{{ masteredCount }}</div>
              <div class="text-xs text-slate-400">认识</div>
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
              继续学习
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
          :current="currentIndex + 1"
          :total="cards.length"
          class="flex-1"
          @rate="handleRate"
        />
      </template>
    </div>
    <BottomNav />
  </div>
</template>
