<script setup>
import { computed, ref, watch } from 'vue'
import { Loader, MoreVertical, Volume2, VolumeX } from 'lucide-vue-next'
import { speakWord } from '../platform/tts.js'

const studyHeroArt = new URL('../assets/hero/study-hero-bg.png', import.meta.url).href
const studyDeckIcon = new URL('../assets/icons/study-deck-icon.svg', import.meta.url).href
const studyKnownIcon = new URL('../assets/icons/study-rate-known.svg', import.meta.url).href
const studyHazyIcon = new URL('../assets/icons/study-rate-hazy.svg', import.meta.url).href
const studyUnknownIcon = new URL('../assets/icons/study-rate-unknown.svg', import.meta.url).href
const studyPauseIcon = new URL('../assets/icons/study-progress-pause.svg', import.meta.url).href
const studyPlayIcon = new URL('../assets/icons/study-progress-play.svg', import.meta.url).href

const props = defineProps({
  card: { type: Object, required: true },
  current: { type: Number, default: 0 },
  total: { type: Number, default: 0 },
  practiceMode: { type: Boolean, default: false },
  paused: { type: Boolean, default: false },
  estimatedMinutes: { type: Number, default: 1 },
  deckName: { type: String, default: '' }
})

const emit = defineEmits(['rate', 'toggle-pause', 'continue-study'])

const revealed = ref(false)
const ttsState = ref('idle') // idle | loading | playing | unavailable
const progressPercent = computed(() => {
  if (!props.total) return 0
  return Math.max(6, Math.round((props.current / props.total) * 100))
})

watch(
  () => props.card?.id,
  () => {
    revealed.value = false
  },
  { immediate: true }
)

function revealAnswer() {
  if (props.paused) return
  revealed.value = true
}

function handleRate(quality) {
  if (props.paused) return
  if (!revealed.value) {
    revealed.value = true
    return
  }
  emit('rate', quality)
}

function speak(event) {
  event.stopPropagation()
  if (props.paused) return
  speakWord(props.card.word, {
    onStateChange: state => { ttsState.value = state }
  }).catch(() => {
    ttsState.value = 'unavailable'
  })
}
</script>

<template>
  <div class="flex w-full flex-1 flex-col select-none">
    <div
      class="soft-panel flashcard-panel relative flex min-h-[378px] flex-1 flex-col overflow-hidden rounded-[24px] px-[1.05rem] py-[0.98rem] text-left"
      :class="{ 'flashcard-panel-paused': paused }"
      :style="{ '--hero-image': `url(${studyHeroArt})` }"
      @click="revealAnswer"
      @keydown.space.prevent="revealAnswer"
      tabindex="0"
    >
      <div class="flashcard-header">
        <div class="flashcard-deck-chip">
          <img class="flashcard-deck-chip-icon" :src="studyDeckIcon" alt="" aria-hidden="true" />
          <span>{{ deckName || card.deckName || (practiceMode ? '自由练习' : '今日学习') }}</span>
        </div>
        <button class="flashcard-menu-button" type="button" @click.stop>
          <MoreVertical class="h-5 w-5" />
        </button>
      </div>

      <div class="flashcard-copy">
        <div class="flashcard-word-row">
          <span class="flashcard-speak-anchor" aria-hidden="true" />
          <h2 class="flashcard-word">{{ card.word }}</h2>
          <button
            class="flashcard-speak-button"
            @click="speak"
            :title="ttsState === 'unavailable' ? 'TTS 不可用' : '发音'"
            :disabled="ttsState === 'loading' || paused"
          >
            <Loader v-if="ttsState === 'loading'" class="h-4 w-4 animate-spin" />
            <VolumeX v-else-if="ttsState === 'unavailable'" class="h-4 w-4 text-slate-300" />
            <Volume2 v-else class="h-4 w-4" />
          </button>
        </div>

        <div class="flashcard-answer-slot">
          <p class="flashcard-reveal-hint" :class="{ 'flashcard-reveal-hint-hidden': revealed }">点击卡片查看答案</p>

          <div class="flashcard-answer" :class="{ 'flashcard-answer-revealed': revealed }" aria-hidden="!revealed">
            <div class="flashcard-inflections">
              <p
                v-for="(inflection, index) in card.inflections"
                :key="`${card.id}-inflection-${index}`"
                class="flashcard-inflection-line"
              >
                {{ inflection }}
              </p>
              <p v-if="!card.inflections || !card.inflections.length" class="flashcard-inflection-line flashcard-inflection-line-muted">
                --
              </p>
            </div>

            <p v-if="card.definition" class="flashcard-definition">
              {{ card.definition }}
            </p>
            <p v-else class="flashcard-definition flashcard-definition-empty">暂无释义</p>
          </div>
        </div>

        <p v-if="ttsState === 'unavailable'" class="flashcard-tts-note">发音不可用，请检查网络或 TTS 设置</p>
      </div>

      <div class="flashcard-rate-grid" @click.stop>
        <button
          class="flashcard-rate-button flashcard-rate-button-unknown"
          :disabled="paused"
          @click="handleRate(0)"
        >
          <img class="flashcard-rate-icon" :src="studyUnknownIcon" alt="" aria-hidden="true" />
          <span class="flashcard-rate-label flashcard-rate-label-unknown">不认识</span>
        </button>
        <button
          class="flashcard-rate-button flashcard-rate-button-hazy"
          :disabled="paused"
          @click="handleRate(3)"
        >
          <img class="flashcard-rate-icon" :src="studyHazyIcon" alt="" aria-hidden="true" />
          <span class="flashcard-rate-label flashcard-rate-label-hazy">模糊</span>
        </button>
        <button
          class="flashcard-rate-button flashcard-rate-button-known"
          :disabled="paused"
          @click="handleRate(5)"
        >
          <img class="flashcard-rate-icon" :src="studyKnownIcon" alt="" aria-hidden="true" />
          <span class="flashcard-rate-label flashcard-rate-label-known">认识</span>
        </button>
      </div>

      <div class="flashcard-progress-card" @click.stop>
        <div class="flashcard-progress-top">
          <span class="flashcard-progress-count">第 {{ current }} / {{ total }} 张</span>
        </div>
        <div class="flashcard-progress-main">
          <button
            class="flashcard-progress-toggle"
            :class="{ 'flashcard-progress-toggle-paused': paused }"
            type="button"
            @click="paused ? emit('continue-study') : emit('toggle-pause')"
            :title="paused ? '继续学习' : '暂停'"
          >
            <img
              class="flashcard-progress-icon"
              :src="paused ? studyPlayIcon : studyPauseIcon"
              alt=""
              aria-hidden="true"
            />
          </button>
          <div class="progress-track flashcard-progress-track">
            <div class="progress-fill" :style="{ width: `${progressPercent}%` }" />
          </div>
        </div>
        <span class="flashcard-progress-estimate">预计 {{ estimatedMinutes }} 分钟完成</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.flashcard-panel {
  position: relative;
  border-color: rgba(214, 224, 244, 0.7);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.98) 0%, rgba(247, 250, 255, 0.96) 100%);
  box-shadow: 0 14px 28px rgba(73, 100, 158, 0.05);
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
}

.flashcard-panel::before {
  content: "";
  position: absolute;
  inset: 0;
  background-image: var(--hero-image);
  background-position: 78% 54%;
  background-repeat: no-repeat;
  background-size: 108% auto;
  opacity: 0.34;
  filter: saturate(0.84) contrast(0.94) brightness(1.04);
  transform: translateY(-72px);
  pointer-events: none;
}

.flashcard-panel::after {
  content: "";
  position: absolute;
  inset: 0;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.44) 0%, rgba(255, 255, 255, 0.68) 22%, rgba(255, 255, 255, 0.9) 56%, rgba(255, 255, 255, 0.96) 100%);
  pointer-events: none;
}

.flashcard-panel-paused .flashcard-copy,
.flashcard-panel-paused .flashcard-rate-grid {
  opacity: 0.58;
}

.flashcard-header,
.flashcard-copy,
.flashcard-rate-grid,
.flashcard-progress-card {
  position: relative;
  z-index: 1;
}

.flashcard-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.flashcard-deck-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.56rem;
  color: #18356f;
  font-size: 0.92rem;
  font-weight: 900;
}

.flashcard-deck-chip-icon {
  width: 1.56rem;
  height: 1.56rem;
  flex: 0 0 auto;
}

.flashcard-menu-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  border: 0;
  background: transparent;
  color: #7f93bd;
}

.flashcard-copy {
  display: flex;
  max-width: min(18.5rem, calc(100% - 2.4rem));
  flex: 1 1 auto;
  flex-direction: column;
  align-items: center;
  margin-top: 1.8rem;
  text-align: center;
}

.flashcard-word-row {
  display: inline-grid;
  grid-template-columns: 2.32rem auto 2.32rem;
  align-items: center;
  column-gap: 0.42rem;
  width: 100%;
  justify-content: center;
}

.flashcard-word {
  margin: 0;
  color: #182f6c;
  font-size: clamp(2.8rem, 7vw, 3.4rem);
  line-height: 0.9;
  font-weight: 950;
  letter-spacing: -0.06em;
  text-align: center;
}

.flashcard-speak-anchor {
  width: 2.32rem;
  height: 2.32rem;
  opacity: 0;
}

.flashcard-speak-button {
  display: grid;
  width: 2.32rem;
  height: 2.32rem;
  flex: 0 0 auto;
  place-items: center;
  border: 0;
  border-radius: 999px;
  background: rgba(232, 240, 255, 0.92);
  color: #2c69f9;
  box-shadow:
    0 8px 18px rgba(68, 103, 191, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.flashcard-reveal-hint {
  position: absolute;
  inset: 0;
  margin: 0;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  color: #8fa0be;
  font-size: 0.76rem;
  font-weight: 700;
  line-height: 1.5;
  padding-top: 1rem;
  transition: opacity 0.18s ease-out;
}

.flashcard-reveal-hint-hidden {
  opacity: 0;
  pointer-events: none;
}

.flashcard-answer-slot {
  position: relative;
  width: 100%;
  margin-top: 1rem;
}

.flashcard-answer {
  position: relative;
  width: 100%;
  opacity: 0;
  visibility: hidden;
  transition: opacity 0.24s ease-out;
}

.flashcard-answer-revealed {
  opacity: 1;
  visibility: visible;
}

.flashcard-inflections {
  display: flex;
  flex-direction: column;
  gap: 0.22rem;
  width: 100%;
}

.flashcard-inflection-line {
  margin: 0;
  width: 100%;
  color: #29406f;
  font-size: 1.02rem;
  line-height: 1.34;
  font-weight: 700;
  word-break: break-word;
  text-align: center;
}

.flashcard-inflection-line-muted {
  color: #9aabc8;
  font-weight: 700;
}

.flashcard-tts-note {
  margin: 0.6rem 0 0;
  color: #99a8c2;
  font-size: 0.72rem;
  line-height: 1.45;
}

.flashcard-definition {
  margin: 1rem 0 0;
  width: 100%;
  color: #122f6e;
  font-size: 1.08rem;
  line-height: 1.42;
  font-weight: 850;
  word-break: break-word;
  text-align: center;
}

.flashcard-definition-empty {
  color: #95a5c1;
}

.flashcard-rate-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.6rem;
  margin-top: auto;
  padding-top: 1.05rem;
}

.flashcard-rate-button {
  display: flex;
  min-height: 3rem;
  flex-direction: row;
  align-items: center;
  justify-content: center;
  gap: 0.28rem;
  border-radius: 0.74rem;
  background: rgba(255, 255, 255, 0.94);
  padding: 0 0.08rem;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.flashcard-rate-button:disabled {
  opacity: 0.5;
  filter: grayscale(0.12);
}

.flashcard-rate-button-known {
  border: 1px solid rgba(155, 229, 206, 0.9);
  background: rgba(243, 255, 251, 0.96);
}

.flashcard-rate-button-hazy {
  border: 1px solid rgba(255, 221, 165, 0.96);
  background: rgba(255, 250, 240, 0.96);
}

.flashcard-rate-button-unknown {
  border: 1px solid rgba(255, 201, 197, 0.96);
  background: rgba(255, 246, 246, 0.96);
}

.flashcard-rate-icon {
  width: 1.5rem;
  height: 1.5rem;
  flex: 0 0 auto;
}

.flashcard-rate-label {
  font-size: 1rem;
  font-weight: 900;
}

.flashcard-rate-label-known {
  color: #13b98f;
}

.flashcard-rate-label-hazy {
  color: #ff9c10;
}

.flashcard-rate-label-unknown {
  color: #ff4f44;
}

.flashcard-progress-card {
  margin-top: 1rem;
  border: 1px solid rgba(229, 235, 246, 0.96);
  border-radius: 0.84rem;
  background: rgba(255, 255, 255, 0.88);
  padding: 0.52rem 0.62rem 0.56rem;
  box-shadow:
    0 10px 18px rgba(74, 103, 163, 0.025),
    inset 0 1px 0 rgba(255, 255, 255, 0.96);
}

.flashcard-progress-top {
  margin-bottom: 0.34rem;
}

.flashcard-progress-main {
  display: flex;
  align-items: center;
  gap: 0.52rem;
}

.flashcard-progress-count {
  color: #566c9b;
  font-size: 0.8rem;
  font-weight: 900;
}

.flashcard-progress-estimate {
  display: block;
  color: #8a9cbc;
  font-size: 0.68rem;
  font-weight: 700;
  margin-top: 0.36rem;
}

.flashcard-progress-toggle {
  display: grid;
  width: 1.66rem;
  height: 1.66rem;
  flex: 0 0 auto;
  place-items: center;
  border: 0;
  border-radius: 0.58rem;
  background: linear-gradient(180deg, rgba(240, 245, 255, 0.98) 0%, rgba(231, 238, 252, 0.98) 100%);
  padding: 0;
  color: #6880ad;
  box-shadow:
    0 6px 12px rgba(74, 103, 163, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.96);
}

.flashcard-progress-toggle-paused {
  background: linear-gradient(135deg, #3f78ff 0%, #245dff 100%);
  color: white;
  box-shadow:
    0 8px 16px rgba(54, 94, 210, 0.16),
    inset 0 1px 0 rgba(255, 255, 255, 0.22);
}

.flashcard-progress-track {
  flex: 1;
  height: 0.34rem;
  margin-top: 0;
}

.flashcard-progress-icon {
  width: 0.78rem;
  height: 0.78rem;
  flex: 0 0 auto;
}
</style>
