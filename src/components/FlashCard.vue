<script setup>
import { ref, watch } from 'vue'
import { Volume2 } from 'lucide-vue-next'

const props = defineProps({
  card: { type: Object, required: true }
})

const emit = defineEmits(['rate'])

const flipped = ref(false)

watch(() => props.card?.id, () => {
  flipped.value = false
})

function toggleFlip() {
  flipped.value = !flipped.value
}

function speak() {
  if ('speechSynthesis' in window) {
    const utterance = new SpeechSynthesisUtterance(props.card.word)
    utterance.lang = 'en-US'
    utterance.rate = 0.9
    speechSynthesis.speak(utterance)
  }
}
</script>

<template>
  <div
    class="relative w-full max-w-sm mx-auto cursor-pointer select-none"
    style="perspective: 1000px; aspect-ratio: 3 / 2"
    @click="toggleFlip"
    @keydown.space.prevent="toggleFlip"
    tabindex="0"
  >
    <div
      class="relative w-full h-full transition-transform duration-500"
      :style="{
        transformStyle: 'preserve-3d',
        transform: flipped ? 'rotateY(180deg)' : 'rotateY(0deg)'
      }"
    >
      <!-- Front -->
      <div
        class="absolute inset-0 bg-white rounded-2xl shadow-lg flex flex-col items-center justify-center p-8 backface-hidden"
        :style="{ backfaceVisibility: 'hidden' }"
      >
        <h2 class="text-4xl font-bold text-gray-800 mb-3">{{ card.word }}</h2>
        <div v-if="card.inflections && card.inflections.length" class="text-lg text-gray-400 mb-4">
          {{ card.inflections.join(' · ') }}
        </div>
        <button
          class="p-2 rounded-full hover:bg-gray-100 text-gray-400 hover:text-blue-500 transition-colors"
          @click.stop="speak"
          title="发音"
        >
          <Volume2 class="w-6 h-6" />
        </button>
        <p class="absolute bottom-4 text-xs text-gray-300">点击翻转</p>
      </div>

      <!-- Back -->
      <div
        class="absolute inset-0 bg-white rounded-2xl shadow-lg flex flex-col items-center justify-center p-8 backface-hidden"
        :style="{ backfaceVisibility: 'hidden', transform: 'rotateY(180deg)' }"
      >
        <h2 class="text-3xl font-bold text-gray-800 mb-2">{{ card.word }}</h2>
        <div v-if="card.inflections && card.inflections.length" class="text-sm text-gray-400 mb-4">
          {{ card.inflections.join(' · ') }}
        </div>
        <p v-if="card.definition" class="text-2xl text-gray-600 mb-6 font-medium">{{ card.definition }}</p>
        <p v-else class="text-lg text-gray-400 mb-6 italic">暂无释义</p>

        <div class="flex gap-3 w-full px-4" @click.stop>
          <button
            class="flex-1 py-3 rounded-xl text-white font-medium bg-red-500 hover:bg-red-600 active:scale-95 transition-all cursor-pointer"
            @click="emit('rate', 0)"
          >
            忘了
          </button>
          <button
            class="flex-1 py-3 rounded-xl text-white font-medium bg-amber-400 hover:bg-amber-500 active:scale-95 transition-all cursor-pointer"
            @click="emit('rate', 3)"
          >
            模糊
          </button>
          <button
            class="flex-1 py-3 rounded-xl text-white font-medium bg-green-500 hover:bg-green-600 active:scale-95 transition-all cursor-pointer"
            @click="emit('rate', 5)"
          >
            掌握
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
