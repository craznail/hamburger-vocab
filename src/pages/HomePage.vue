<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '../stores/useAppStore'
import { BookOpen, Plus, Book, CheckCircle, Clock, ArrowRight, Download, Volume2, Play, Loader, VolumeX } from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
import FileUpload from '../components/FileUpload.vue'
import { speakWord } from '../platform/tts.js'
import ImportPreview from '../components/ImportPreview.vue'

const router = useRouter()
const store = useAppStore()
import { downloadDB } from '../api/card'

const ttsWord = ref('')
const ttsState = ref('idle')

const showPreview = ref(false)
const previewFile = ref('')
const previewResult = ref(null)

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

async function onFileSelected(fileName, text) {
  const result = await store.importFile(fileName, text)
  if (!result.success) {
    alert(result.error)
    return
  }
  // Navigate to study page after import
  router.push({ name: 'Study', query: { deckId: result.deckId } })
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

function getTotalMasteredRatio(deck) {
  if (deck.wordCount === 0) return 0
  return Math.round((deck.masteredCount / deck.wordCount) * 100)
}
</script>

<template>
  <div class="min-h-screen flex flex-col bg-[#fafafa]">
    <NavBar :showBack="false">
      <template #left>
        <h1 class="text-lg font-bold text-gray-800">单词卡片</h1>
      </template>
    </NavBar>

    <div class="flex-1 px-5 py-6">
      <p class="text-sm text-gray-400 mb-6">上传 txt 文件，以卡片方式背单词</p>

      <!-- Today's Review -->
      <div class="bg-white rounded-xl shadow-sm border border-gray-100 p-6 mb-6">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2 text-gray-600">
            <Clock class="w-5 h-5" />
            <span class="font-medium">今日待复习</span>
          </div>
        </div>
        <div class="text-5xl font-bold text-blue-600 mb-2">
          {{ store.todayCount }}
        </div>
        <p class="text-sm text-gray-400 mb-4">张卡片需要今天复习</p>
        <div v-if="store.todayCount > 0" class="flex gap-3">
          <button
            class="flex items-center gap-2 px-5 py-2.5 bg-blue-600 text-white rounded-xl hover:bg-blue-700 transition-colors cursor-pointer"
            @click="goStudy()"
          >
            <BookOpen class="w-4 h-4" />
            开始学习
            <ArrowRight class="w-4 h-4" />
          </button>
          <button
            class="flex items-center gap-2 px-5 py-2.5 border border-blue-200 text-blue-600 rounded-xl hover:bg-blue-50 transition-colors cursor-pointer"
            @click="goDictation"
          >
            <Volume2 class="w-4 h-4" />
            开始听写
          </button>
        </div>
      </div>

      <!-- Upload -->
      <div class="mb-6">
        <FileUpload @file-selected="onFileSelected" />
      </div>

      <div class="bg-white rounded-xl shadow-sm border border-gray-100 p-4 mb-6">
        <div class="flex items-center gap-2 text-gray-600 mb-3">
          <Volume2 class="w-4 h-4" />
          <span class="font-medium text-sm">快速发音测试</span>
        </div>
        <div class="flex gap-2">
          <input
            v-model="ttsWord"
            type="text"
            placeholder="输入单词测试发音"
            class="flex-1 px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-200 focus:border-blue-400"
            @keyup.enter="testTTS"
          />
          <button
            class="inline-flex items-center gap-1.5 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm cursor-pointer disabled:opacity-50"
            :disabled="ttsState === 'loading' || !ttsWord.trim()"
            @click="testTTS"
          >
            <Loader v-if="ttsState === 'loading'" class="w-4 h-4 animate-spin" />
            <Play v-else class="w-4 h-4" />
            发音
          </button>
        </div>
        <p v-if="ttsState === 'unavailable'" class="text-xs text-red-400 mt-2">发音不可用</p>
        <p v-if="ttsError" class="text-xs text-gray-400 mt-1 break-all">{{ ttsError }}</p>
        <p v-else-if="ttsState === 'playing'" class="text-xs text-green-500 mt-2">正在播放...</p>
      </div>

      <!-- Decks List -->
      <div v-if="store.decks.length > 0">
        <h2 class="text-lg font-semibold text-gray-700 mb-3">词库列表</h2>
        <div class="space-y-3">
          <div
            v-for="deck in store.sortedDecks"
            :key="deck.id"
            class="bg-white rounded-xl shadow-sm border border-gray-100 p-4 hover:shadow-md transition-shadow cursor-pointer"
            @click="goDeckDetail(deck.id)"
          >
            <div class="flex items-start justify-between mb-2">
              <div class="flex items-center gap-2">
                <Book class="w-5 h-5 text-blue-500" />
                <span class="font-medium text-gray-800">{{ deck.name }}</span>
              </div>
              <div class="flex items-center gap-1 text-sm">
                <CheckCircle v-if="deck.dueCount > 0" class="w-4 h-4 text-amber-500" />
                <span v-if="deck.dueCount > 0" class="text-amber-600 font-medium">{{ deck.dueCount }} 待复习</span>
              </div>
            </div>

            <div class="w-full bg-gray-100 rounded-full h-1.5 mb-2">
              <div
                class="bg-green-400 h-1.5 rounded-full transition-all"
                :style="{ width: `${getTotalMasteredRatio(deck)}%` }"
              />
            </div>

            <div class="flex gap-4 text-xs text-gray-400">
              <span>共 {{ deck.wordCount }} 词</span>
              <span>已掌握 {{ deck.masteredCount }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else class="text-center py-12">
        <Book class="w-16 h-16 mx-auto mb-4 text-gray-200" />
        <p class="text-gray-400 mb-2">还没有词库</p>
        <p class="text-sm text-gray-300">上传一个 txt 文件开始背单词吧</p>
      </div>

      <!-- Footer -->
      <div class="mt-8 pt-6 border-t border-gray-100 text-center">
        <button
          class="text-xs text-gray-300 hover:text-gray-500 transition-colors cursor-pointer inline-flex items-center gap-1"
          @click="downloadDB"
        >
          <Download class="w-3 h-3" />
          导出数据 (.sqlite)
        </button>
      </div>
    </div>
  </div>
</template>
