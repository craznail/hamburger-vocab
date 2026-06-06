<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore } from '../stores/useAppStore'
import { ArrowLeft, CheckCircle, XCircle, BarChart3 } from 'lucide-vue-next'
import FlashCard from '../components/FlashCard.vue'
import ProgressBar from '../components/ProgressBar.vue'

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
  current: currentIndex.value + 1,
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

onMounted(() => {
  loadCards()
})

function loadCards() {
  cards.value = store.getTodayLearningCards(deckId.value)
  if (cards.value.length === 0) {
    sessionDone.value = true
  }
}

function handleRate(quality) {
  if (!currentCard.value) return

  const card = currentCard.value
  store.rateCard(card.id, quality, {
    ef: card.ef,
    interval: card.interval,
    repetitions: card.repetitions
  })

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

function continueStudy() {
  // Refresh cards and start again
  sessionDone.value = false
  currentIndex.value = 0
  sessionResults.value = []
  loadCards()
}
</script>

<template>
  <div class="min-h-screen flex flex-col">
    <!-- Header -->
    <header class="flex items-center justify-between px-4 py-3 border-b border-gray-100">
      <button
        class="flex items-center gap-1.5 text-gray-500 hover:text-gray-700 transition-colors cursor-pointer"
        @click="goHome"
      >
        <ArrowLeft class="w-5 h-5" />
        <span class="text-sm">返回</span>
      </button>
      <div v-if="!sessionDone" class="text-sm text-gray-400">
        {{ progress.current }} / {{ progress.total }}
      </div>
    </header>

    <!-- Content -->
    <div class="flex-1 flex flex-col items-center justify-center px-4 py-6">
      <!-- Session Done -->
      <div v-if="sessionDone" class="text-center max-w-sm w-full">
        <div v-if="cards.length === 0" class="py-12">
          <CheckCircle class="w-16 h-16 mx-auto mb-4 text-green-400" />
          <h2 class="text-xl font-semibold text-gray-700 mb-2">今日无待复习</h2>
          <p class="text-gray-400 mb-6">所有卡片已复习完毕，明天再来吧</p>
          <button
            class="px-6 py-2.5 bg-blue-600 text-white rounded-xl hover:bg-blue-700 transition-colors cursor-pointer"
            @click="goHome"
          >
            返回首页
          </button>
        </div>
        <div v-else class="py-6">
          <BarChart3 class="w-16 h-16 mx-auto mb-4 text-blue-400" />
          <h2 class="text-xl font-semibold text-gray-700 mb-1">今日学习完成</h2>
          <p class="text-gray-400 mb-6">共复习 {{ cards.length }} 张卡片</p>

          <div class="flex justify-center gap-6 mb-6">
            <div class="text-center">
              <div class="text-2xl font-bold text-green-500">{{ masteredCount }}</div>
              <div class="text-xs text-gray-400">掌握</div>
            </div>
            <div class="text-center">
              <div class="text-2xl font-bold text-amber-400">{{ hazyCount }}</div>
              <div class="text-xs text-gray-400">模糊</div>
            </div>
            <div class="text-center">
              <div class="text-2xl font-bold text-red-500">{{ forgotCount }}</div>
              <div class="text-xs text-gray-400">忘了</div>
            </div>
          </div>

          <div class="flex gap-3 justify-center">
            <button
              class="px-6 py-2.5 bg-blue-600 text-white rounded-xl hover:bg-blue-700 transition-colors cursor-pointer"
              @click="continueStudy"
            >
              继续学习
            </button>
            <button
              class="px-6 py-2.5 border border-gray-300 text-gray-600 rounded-xl hover:bg-gray-50 transition-colors cursor-pointer"
              @click="goHome"
            >
              返回首页
            </button>
          </div>
        </div>
      </div>

      <!-- Active Study -->
      <template v-else-if="currentCard">
        <ProgressBar
          :current="currentIndex + 1"
          :total="cards.length"
          class="mb-6 max-w-sm"
        />
        <FlashCard
          :key="currentCard.id"
          :card="currentCard"
          @rate="handleRate"
        />
      </template>
    </div>
  </div>
</template>
