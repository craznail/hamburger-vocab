<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore } from '../stores/useAppStore'
import { Book, Search, Trash2, Play, CheckCircle, Clock, Loader, Volume2, VolumeX } from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
import { speakWord } from '../platform/tts.js'

const route = useRoute()
const router = useRouter()
const store = useAppStore()

const deck = ref(null)
const cards = ref([])
const stats = ref({ total: 0, mastered: 0, due: 0 })
const searchQuery = ref('')
const activeFilter = ref('all')
const loading = ref(true)
const ttsPlayingWord = ref(null) // the word currently being spoken
const ttsUnavailable = ref(false)

onMounted(async () => {
  try {
    const deckId = route.params.id
    deck.value = await store.getDeckInfo(deckId)
    if (!deck.value) {
      router.push({ name: 'Home' })
      return
    }
    cards.value = await store.getCardsForDeck(deckId)
    stats.value = await store.getDeckStats(deckId)
  } finally {
    loading.value = false
  }
})

const filteredCards = computed(() => {
  let result = cards.value
  const now = new Date()
  const today = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`
  if (activeFilter.value === 'due') {
    result = result.filter(c => c.nextReview <= today)
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

function goDictation() {
  router.push({ name: 'Dictation', query: { deckId: route.params.id } })
}

async function confirmDelete() {
  if (confirm('确定要删除这个词库吗？所有单词数据将丢失。')) {
    await store.removeDeck(route.params.id)
    router.push({ name: 'Home' })
  }
}

function speak(word) {
  speakWord(word, {
    onStateChange: state => {
      ttsPlayingWord.value = state === 'idle' || state === 'unavailable' ? null : word
    }
  }).catch(() => {
    ttsPlayingWord.value = null
    ttsUnavailable.value = true
    setTimeout(() => { ttsUnavailable.value = false }, 3000)
  })
}
</script>

<template>
  <div v-if="loading" class="flex items-center justify-center min-h-screen">
    <p class="text-gray-400">加载中...</p>
  </div>
  <div v-else class="min-h-screen flex flex-col">
    <NavBar @back="goBack">
      <template #right>
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
            v-if="stats.due > 0"
            class="flex items-center gap-1.5 px-4 py-2 border border-blue-200 text-blue-600 rounded-xl hover:bg-blue-50 transition-colors text-sm cursor-pointer"
            @click="goDictation"
          >
            <Volume2 class="w-4 h-4" />
            听写
          </button>
          <button
            class="p-2 text-gray-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors cursor-pointer"
            @click="confirmDelete"
            title="删除词库"
          >
            <Trash2 class="w-5 h-5" />
          </button>
        </div>
      </template>
    </NavBar>

    <div class="flex-1 px-5 py-6">
      <h1 class="text-xl font-bold text-gray-800 flex items-center gap-2 mb-6">
        <Book class="w-5 h-5 text-blue-500" />
        {{ deck?.name }}
      </h1>

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

      <div class="flex gap-2 mb-4">
        <button
          class="px-3 py-1.5 rounded-lg text-sm transition-colors cursor-pointer"
          :class="activeFilter === 'all' ? 'bg-blue-100 text-blue-700' : 'bg-gray-100 text-gray-500 hover:bg-gray-200'"
          @click="activeFilter = 'all'"
        >全部</button>
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

      <!-- TTS unavailable hint -->
      <p v-if="ttsUnavailable" class="text-xs text-gray-400 mb-2 text-center">
        发音不可用，请检查网络或 TTS 设置
      </p>

      <div class="space-y-1">
        <div
          v-for="card in filteredCards"
          :key="card.id"
          class="flex items-center justify-between px-4 py-3 bg-white rounded-lg hover:bg-gray-50 transition-colors border border-gray-100"
        >
          <div class="flex items-center gap-3">
            <button
              class="p-1 transition-colors cursor-pointer"
              :class="ttsPlayingWord === card.word ? 'text-blue-500' : 'text-gray-300 hover:text-blue-500'"
              @click="speak(card.word)"
              :title="ttsPlayingWord === card.word ? '播放中...' : '发音'"
              :disabled="ttsPlayingWord === card.word"
            >
              <Loader v-if="ttsPlayingWord === card.word" class="w-4 h-4 animate-spin" />
              <VolumeX v-else-if="ttsUnavailable && ttsPlayingWord === null" class="w-4 h-4" />
              <Play v-else class="w-4 h-4" />
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

      <div v-if="filteredCards.length === 0 && cards.length > 0" class="text-center py-12 text-gray-400">
        没有匹配的单词
      </div>
      <div v-if="cards.length === 0" class="text-center py-12 text-gray-400">
        词库为空
      </div>
    </div>
  </div>
</template>
