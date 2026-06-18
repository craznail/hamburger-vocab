<script setup>
import { computed, ref } from 'vue'
import { Heart, Loader, Star, Trees, Volume2, VolumeX } from 'lucide-vue-next'
import { speakWord } from '../platform/tts.js'
import { computeNextReview } from '../utils/sm2'

const props = defineProps({
  card: { type: Object, required: true },
  current: { type: Number, default: 0 },
  total: { type: Number, default: 0 },
  practiceMode: { type: Boolean, default: false }
})

const emit = defineEmits(['rate'])

const revealed = ref(false)
const ttsState = ref('idle') // idle | loading | playing | unavailable
const intervals = computed(() => ({
  forgot: computeNextReview(0, props.card).interval,
  hazy: computeNextReview(3, props.card).interval,
  mastered: computeNextReview(5, props.card).interval
}))

function toggleReveal() {
  revealed.value = !revealed.value
}

function speak(event) {
  event.stopPropagation()
  speakWord(props.card.word, {
    onStateChange: state => { ttsState.value = state }
  }).catch(() => {
    ttsState.value = 'unavailable'
  })
}
</script>

<template>
  <div
    class="flex w-full flex-1 flex-col select-none"
    @click="toggleReveal"
    @keydown.space.prevent="toggleReveal"
    tabindex="0"
  >
      <div class="soft-panel relative flex min-h-[390px] flex-1 flex-col justify-center overflow-hidden rounded-[28px] px-6 py-9 text-center">
        <div class="absolute bottom-[-2.4rem] right-[-1.1rem] h-40 w-40 rotate-12 rounded-[2.4rem] bg-[#eaf1ff]/95" />
        <div class="absolute bottom-10 right-10 h-20 w-20 rounded-[1.65rem] border border-blue-100 bg-white/74" />
        <div class="absolute left-6 top-6 h-10 w-10 rounded-2xl border border-blue-100 bg-white/70" />
        <span class="mx-auto mb-7 rounded-xl bg-[#eff3ff] px-3 py-1.5 text-xs font-black text-blue-600">{{ practiceMode ? '自由练习' : '今日学习' }}</span>

        <div class="relative mb-4 flex items-center justify-center gap-2">
          <h2 class="max-w-[260px] text-[2.15rem] font-black leading-snug text-ink">{{ card.word }}</h2>
          <button
            class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-blue-50 text-blue-500 transition-colors hover:bg-blue-100"
            @click="speak"
            :title="ttsState === 'unavailable' ? 'TTS 不可用' : '发音'"
            :disabled="ttsState === 'loading'"
          >
            <Loader v-if="ttsState === 'loading'" class="h-4 w-4 animate-spin" />
            <VolumeX v-else-if="ttsState === 'unavailable'" class="h-4 w-4 text-slate-300" />
            <Volume2 v-else class="h-4 w-4" />
          </button>
        </div>

        <p v-if="!revealed" class="relative text-sm font-semibold text-slate-400">点击卡片查看答案</p>
        <p v-if="ttsState === 'unavailable'" class="relative mt-2 text-xs text-slate-400">发音不可用，请检查网络或 TTS 设置</p>

        <transition name="reveal">
          <div v-if="revealed" class="relative flex flex-col items-center">
            <div
              v-if="card.inflections && card.inflections.length"
              class="mb-3 text-sm font-semibold text-blue-500"
            >
              {{ card.inflections.join(' · ') }}
            </div>
            <p v-if="card.definition" class="px-2 text-lg leading-relaxed text-slate-600">
              {{ card.definition }}
            </p>
            <p v-else class="text-base text-slate-400">暂无释义</p>
          </div>
        </transition>
      </div>

      <div class="grid grid-cols-3 gap-3 pt-5" @click.stop>
        <button
          class="flex min-h-[86px] flex-col items-center justify-center gap-1 rounded-[22px] border border-red-100 bg-white text-[#ef665d] shadow-[0_12px_24px_rgba(120,68,68,0.06)] disabled:grayscale disabled:opacity-45"
          :disabled="!revealed"
          @click="emit('rate', 0)"
        >
          <Heart class="h-6 w-6" />
          <span class="text-sm font-black">忘记</span>
          <span class="text-[10px] text-slate-400">{{ practiceMode ? '仅记录练习' : `${intervals.forgot} 天后复习` }}</span>
        </button>
        <button
          class="flex min-h-[86px] flex-col items-center justify-center gap-1 rounded-[22px] border border-amber-100 bg-white text-[#d99428] shadow-[0_12px_24px_rgba(112,88,44,0.06)] disabled:grayscale disabled:opacity-45"
          :disabled="!revealed"
          @click="emit('rate', 3)"
        >
          <Star class="h-6 w-6" />
          <span class="text-sm font-black">模糊</span>
          <span class="text-[10px] text-slate-400">{{ practiceMode ? '仅记录练习' : `${intervals.hazy} 天后复习` }}</span>
        </button>
        <button
          class="flex min-h-[86px] flex-col items-center justify-center gap-1 rounded-[22px] bg-[#e8f8f4] text-[#249b82] shadow-[0_12px_24px_rgba(42,138,117,0.08)] disabled:grayscale disabled:opacity-45"
          :disabled="!revealed"
          @click="emit('rate', 5)"
        >
          <Trees class="h-6 w-6" />
          <span class="text-sm font-black">已掌握</span>
          <span class="text-[10px] text-[#5aa998]">{{ practiceMode ? '仅记录练习' : `${intervals.mastered} 天后复习` }}</span>
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
