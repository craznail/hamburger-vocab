<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore } from '../stores/useAppStore'
import { ArrowLeft, Book, Search, Trash2, Play, CheckCircle, Clock } from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()
const store = useAppStore()

const deck = ref(null)
const cards = ref([])
const stats = ref({ total: 0, mastered: 0, due: 0 })
const searchQuery = ref('')
const activeFilter = ref('all') // all | due | mastered

onMounted(() => {
  const deckId = route.params.id
  deck.value = store.getDeckInfo(deckId)
  if (!deck.value) {
    router.push({ name: 'Home' })
    return
  }
  cards.value = store.getCardsForDeck(deckId)
  stats.value = store.getDeckStats(deckId)
})

const filteredCards = computed(() => {
  let result = cards.value
  if (activeFilter.value === 'due') {
    result = result.filter(c => c.nextReview <= new Date().toISOString().slice(0, 10) && c.repetitions < 2)
  } else if (activeFilter.value === 'mastered') {
    result = result.filter(c => c.repetitions >= 2)
  }
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.trim().toLowerCase()
    result = result.filter(c => c.word.toLowerCase().includes(q) || c.definition.toLowerCase().includes(q))
  }
  return result
})

function goBack() {
  router.push({ name: 'Home' })
}

function goStudy() {
  router.push({ name: 'Study', query: { deckId: route.params.id } })
}

function confirmDelete() {
  if (confirm('确定要删除这个词库吗？所有单词数据将丢失。')) {
    store.removeDeck(route.params.id)
    router.push({ name: 'Home' })
  }
}

function speak(word) {
  if ('speechSynthesis' in window) {
    const utterance = new SpeechSynthesisUtterance(word)
    utterance.lang = 'en-US'
    utterance.rate = 0.9
    speechSynthesis.speak(utterance)
  }
}
</script>

<template>
  <div class="max-w-2xl mx-auto px-4 py-8">
    <!-- Header -->
    <div class="flex items-start justify-between mb-6">
      <div>
        <button
          class="flex items-center gap-1.5 text-gray-500 hover:text-gray-700 transition-colors mb-3 cursor-pointer"
          @click="goBack"
        >
          <ArrowLeft class="w-4 h-4" />
          <span class="text-sm">返回</span>
        </button>
        <h1 class="text-xl font-bold text-gray-800 flex items-center gap-2">
          <Book class="w-5 h-5 text-blue-500" />
          {{ deck?.name }}
        </h1>
      </div>
      <div class="flex gap-2">
        <button
          v-if="stats.due > 0"
          class="flex items-center gap-1.5 px-4 py-2 bg-blue-600 text-white rounded-xl hover:bg-blue-700 transition-colors text-sm cursor-pointer"
          @click="goStudy"
        >
          <Play class="w-4 h-4" />
          学习
        </button>
        <button
          class="p-2 text-gray-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors cursor-pointer"
          @click="confirmDelete"
          title="删除词库"
        >
          <Trash2 class="w-5 h-5" />
        </button>
      </div>
    </div>

    <!-- Stats -->
    <div class="flex gap-4 mb-6">
      <div class="flex-1 bg-white rounded-xl shadow-sm border border-gray-100 p-4 text-center">
        <div class="text-2xl font-bold text-gray-700">{{ stats.total }}</div>
        <div class="text-xs text-gray-400">总单词</div>
      </div>
      <div class="flex-1 bg-white rounded-xl shadow-sm border border-gray-100 p-4 text-center">
        <div class="text-2xl font-bold text-green-500">{{ stats.mastered }}</div>
        <div class="text-xs text-gray-400">已掌握</div>
      </div>
      <div class="flex-1 bg-white rounded-xl shadow-sm border border-gray-100 p-4 text-center">
        <div class="text-2xl font-bold text-amber-500">{{ stats.due }}</div>
        <div class="text-xs text-gray-400">待复习</div>
      </div>
    </div>

    <!-- Search + Filter -->
    <div class="flex gap-3 mb-4">
      <div class="flex-1 relative">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="搜索单词..."
          class="w-full pl-9 pr-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-200 focus:border-blue-400"
        />
      </div>
    </div>

    <!-- Filter tabs -->
    <div class="flex gap-2 mb-4">
      <button
        class="px-3 py-1.5 rounded-lg text-sm transition-colors cursor-pointer"
        :class="activeFilter === 'all' ? 'bg-blue-100 text-blue-700' : 'bg-gray-100 text-gray-500 hover:bg-gray-200'"
        @click="activeFilter = 'all'"
      >
        全部
      </button>
      <button
        class="px-3 py-1.5 rounded-lg text-sm flex items-center gap-1 transition-colors cursor-pointer"
        :class="activeFilter === 'due' ? 'bg-amber-100 text-amber-700' : 'bg-gray-100 text-gray-500 hover:bg-gray-200'"
        @click="activeFilter = 'due'"
      >
        <Clock class="w-3.5 h-3.5" />
        待复习
      </button>
      <button
        class="px-3 py-1.5 rounded-lg text-sm flex items-center gap-1 transition-colors cursor-pointer"
        :class="activeFilter === 'mastered' ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-500 hover:bg-gray-200'"
        @click="activeFilter = 'mastered'"
      >
        <CheckCircle class="w-3.5 h-3.5" />
        已掌握
      </button>
    </div>

    <!-- Word list -->
    <div class="space-y-1">
      <div
        v-for="card in filteredCards"
        :key="card.id"
        class="flex items-center justify-between px-4 py-3 bg-white rounded-lg hover:bg-gray-50 transition-colors border border-gray-100"
      >
        <div class="flex items-center gap-3">
          <button
            class="p-1 text-gray-300 hover:text-blue-500 transition-colors cursor-pointer"
            @click="speak(card.word)"
            title="发音"
          >
            <Play class="w-4 h-4" />
          </button>
          <div>
            <div class="font-medium text-gray-800">{{ card.word }}</div>
            <div v-if="card.inflections && card.inflections.length" class="text-xs text-gray-400">
              {{ card.inflections.join(' · ') }}
            </div>
          </div>
        </div>
        <div class="flex items-center gap-3">
          <span class="text-sm text-gray-500">{{ card.definition || '-' }}</span>
          <div class="flex items-center">
            <div v-if="card.repetitions >= 2" class="text-green-500" title="已掌握">
              <CheckCircle class="w-4 h-4" />
            </div>
            <div v-else-if="card.nextReview" class="text-amber-400" title="待复习">
              <Clock class="w-4 h-4" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <div v-if="filteredCards.length === 0 && cards.length > 0" class="text-center py-12 text-gray-400">
      没有匹配的单词
    </div>
    <div v-if="cards.length === 0" class="text-center py-12 text-gray-400">
      词库为空
    </div>
  </div>
</template>
