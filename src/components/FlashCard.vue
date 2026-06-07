<script setup>
import { ref } from 'vue'
import { Volume2, Loader, VolumeX } from 'lucide-vue-next'

const props = defineProps({
  card: { type: Object, required: true },
  current: { type: Number, default: 0 },
  total: { type: Number, default: 0 }
})

const emit = defineEmits(['rate'])

const revealed = ref(false)
const ttsState = ref('idle') // idle | loading | playing | unavailable

function toggleReveal() {
  revealed.value = !revealed.value
}

// Check if Web Speech API has usable voices
function hasSpeechVoices() {
  if (!('speechSynthesis' in window)) return false
  try {
    const voices = speechSynthesis.getVoices()
    return voices.length > 0
  } catch {
    return false
  }
}

// Fallback: Google Translate TTS audio
function speakViaAudio(word) {
  ttsState.value = 'loading'
  const url = `https://translate.google.com/translate_tts?ie=UTF-8&client=tw-ob&tl=en&q=${encodeURIComponent(word)}`
  const audio = new Audio(url)
  audio.onplaying = () => { ttsState.value = 'playing' }
  audio.onended = () => { ttsState.value = 'idle' }
  audio.onerror = () => { ttsState.value = 'unavailable' }
  audio.play().catch(() => {
    ttsState.value = 'unavailable'
  })
}

// Try Web Speech API; voices must be available
function speakViaWebSpeech(word) {
  ttsState.value = 'loading'
  const utterance = new SpeechSynthesisUtterance(word)
  utterance.lang = 'en-US'
  utterance.rate = 0.9
  utterance.onstart = () => { ttsState.value = 'playing' }
  utterance.onend = () => { ttsState.value = 'idle' }
  utterance.onerror = () => {
    // Web Speech failed — try audio fallback
    speakViaAudio(word)
  }
  speechSynthesis.speak(utterance)
}

function speak(event) {
  event.stopPropagation()

  if (hasSpeechVoices()) {
    speakViaWebSpeech(props.card.word)
  } else {
    speakViaAudio(props.card.word)
  }
}
</script>

<template>
  <div
    class="relative w-full mx-auto bg-white shadow-lg flex flex-col overflow-hidden cursor-pointer select-none"
    @click="toggleReveal"
    @keydown.space.prevent="toggleReveal"
    tabindex="0"
  >
      <!-- Progress bar - flush top, inside card -->
      <div v-if="total > 0" class="w-full flex-shrink-0 h-1.5 bg-gray-200 overflow-hidden">
        <div
          class="h-full bg-blue-500 transition-all duration-300"
          :style="{ width: `${Math.round((current / total) * 100)}%` }"
        />
      </div>

      <!-- Content area -->
      <div class="flex-1 flex flex-col items-center justify-center px-4 sm:px-6 md:px-8">
        <div class="flex items-center gap-2 sm:gap-3 mb-4 sm:mb-5 md:mb-6">
          <h2 class="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-bold text-gray-800">{{ card.word }}</h2>
          <button
            class="p-2 sm:p-3 rounded-full hover:bg-gray-100 text-gray-400 hover:text-blue-500 transition-colors"
            @click="speak"
            :title="ttsState === 'unavailable' ? 'TTS 不可用' : '发音'"
            :disabled="ttsState === 'loading'"
          >
            <Loader v-if="ttsState === 'loading'" class="w-6 h-6 sm:w-7 sm:h-7 animate-spin" />
            <VolumeX v-else-if="ttsState === 'unavailable'" class="w-6 h-6 sm:w-7 sm:h-7 text-gray-300" />
            <Volume2 v-else class="w-6 h-6 sm:w-7 sm:h-7" />
          </button>
        </div>

        <!-- TTS unavailable hint -->
        <p v-if="ttsState === 'unavailable'" class="text-xs text-gray-400 mt-1">发音不可用，请检查网络或 TTS 设置</p>

        <transition name="reveal">
          <div v-if="revealed" class="flex flex-col items-center">
            <div
              v-if="card.inflections && card.inflections.length"
              class="text-xl sm:text-2xl md:text-3xl lg:text-4xl text-gray-500 mb-3 sm:mb-4 md:mb-5"
            >
              {{ card.inflections.join(' · ') }}
            </div>
            <p
              v-if="card.definition"
              class="text-base sm:text-lg md:text-xl lg:text-2xl text-gray-600 leading-relaxed text-center px-2"
            >
              {{ card.definition }}
            </p>
            <p v-else class="text-base sm:text-lg md:text-xl lg:text-2xl text-gray-400 italic">暂无释义</p>
          </div>
        </transition>
      </div>

      <!-- Rating buttons -->
      <div class="flex gap-3 w-full px-4 sm:px-6 md:px-8 pb-5 sm:pb-7 md:pb-9" @click.stop>
        <button
          class="flex-1 py-2 sm:py-2.5 md:py-3 rounded-xl text-white font-medium text-sm sm:text-base bg-red-500 disabled:bg-gray-300 hover:bg-red-600 disabled:hover:bg-gray-300 active:scale-95 transition-all cursor-pointer disabled:cursor-not-allowed"
          :disabled="!revealed"
          @click="emit('rate', 0)"
        >
          忘了
        </button>
        <button
          class="flex-1 py-2 sm:py-2.5 md:py-3 rounded-xl text-white font-medium text-sm sm:text-base bg-amber-400 disabled:bg-gray-300 hover:bg-amber-500 disabled:hover:bg-gray-300 active:scale-95 transition-all cursor-pointer disabled:cursor-not-allowed"
          :disabled="!revealed"
          @click="emit('rate', 3)"
        >
          模糊
        </button>
        <button
          class="flex-1 py-2 sm:py-2.5 md:py-3 rounded-xl text-white font-medium text-sm sm:text-base bg-green-500 disabled:bg-gray-300 hover:bg-green-600 disabled:hover:bg-gray-300 active:scale-95 transition-all cursor-pointer disabled:cursor-not-allowed"
          :disabled="!revealed"
          @click="emit('rate', 5)"
        >
          掌握
        </button>
      </div>
    </div>
</template>

<style scoped>
.reveal-enter-active {
  transition: all 0.3s ease-out;
}
.reveal-leave-active {
  transition: all 0.2s ease-in;
}
.reveal-enter-from {
  opacity: 0;
  transform: translateY(-6px);
}
.reveal-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
