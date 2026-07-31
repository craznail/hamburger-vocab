<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'
import {
  ArrowLeft,
  Brain,
  CheckCircle,
  ChevronDown,
  Clock3,
  Lightbulb,
  NotebookPen,
  Target,
} from 'lucide-vue-next'
import * as errorApi from '../api/errorItem'
import RichText from '../components/RichText.vue'

const router = useRouter()
const items = ref<errorApi.ErrorItem[]>([])
const index = ref(0)
const revealed = ref(false)
const startedAt = ref(Date.now())
const done = ref(false)
const activeTab = ref<'answer' | 'mistake'>('answer')
const knowledgeExpanded = ref(false)
const knowledgeOverflowing = ref(false)
const localImageFailed = ref(false)
const knowledgeTagsRef = ref<HTMLElement | null>(null)

const current = computed(() => items.value[index.value] || null)
const imageSrc = computed(() => {
  if (current.value?.localImagePath && !localImageFailed.value) {
    return convertFileSrc(current.value.localImagePath)
  }
  return current.value?.remoteImageUrl || ''
})
const knowledgePoints = computed(() => current.value ? errorApi.parseKnowledgePoints(current.value.knowledgePoints) : [])
const progressCurrent = computed(() => {
  if (items.value.length === 0) return 0
  return done.value ? items.value.length : Math.min(index.value + 1, items.value.length)
})
const progressTotal = computed(() => items.value.length)
const progressPercent = computed(() => {
  if (items.value.length === 0) return 100
  return Math.max(8, Math.round((progressCurrent.value / items.value.length) * 100))
})
const hasMistakeContent = computed(() => Boolean(
  current.value?.wrongAnswerText || current.value?.mistakeAnalysis || current.value?.userNotes,
))

onMounted(async () => {
  items.value = await errorApi.getDueErrorItems()
  if (items.value.length === 0) done.value = true
  await nextTick()
  updateKnowledgeOverflow()
})

watch(
  () => [current.value?.id, knowledgePoints.value.join('|')],
  async () => {
    localImageFailed.value = false
    await nextTick()
    updateKnowledgeOverflow()
  },
)

function handleImageError() {
  if (current.value?.localImagePath && !localImageFailed.value) {
    localImageFailed.value = true
  }
}

async function rate(quality: number) {
  if (!current.value) return
  const seconds = Math.max(1, Math.round((Date.now() - startedAt.value) / 1000))
  await errorApi.rateErrorItem(current.value.id, quality, seconds)
  if (index.value < items.value.length - 1) {
    index.value++
    revealed.value = false
    activeTab.value = 'answer'
    knowledgeExpanded.value = false
    knowledgeOverflowing.value = false
    startedAt.value = Date.now()
    requestAnimationFrame(() => updateKnowledgeOverflow())
  } else {
    done.value = true
  }
}

function updateKnowledgeOverflow() {
  const el = knowledgeTagsRef.value
  if (!el) {
    knowledgeOverflowing.value = false
    return
  }

  const chips = Array.from(el.querySelectorAll<HTMLElement>('.review-chip'))
  if (chips.length <= 1) {
    knowledgeOverflowing.value = false
    return
  }

  const firstTop = chips[0]?.offsetTop ?? 0
  knowledgeOverflowing.value = chips.some(chip => chip.offsetTop > firstTop + 1)
  if (!knowledgeOverflowing.value) knowledgeExpanded.value = false
}
</script>

<template>
  <div class="app-page review-page min-h-screen">
    <header class="review-topbar header-safe-top">
      <button class="review-nav-button" type="button" aria-label="返回错题本" @click="router.push({ name: 'ErrorNotebook' })">
        <ArrowLeft class="h-6 w-6" />
      </button>
      <div class="review-title-block">
        <h1 class="review-title">错题复习</h1>
        <p class="review-subtitle">把错因重新学会，推进到长期记忆</p>
      </div>
      <div class="review-progress-pill" :class="{ 'review-progress-pill-complete': done }" aria-label="复习进度">
        <template v-if="progressTotal > 0">
          <div class="review-progress-numbers">
            <strong>{{ progressCurrent }}</strong>
            <span>/ {{ progressTotal }}</span>
          </div>
          <div class="review-progress-track">
            <span class="review-progress-fill" :style="{ width: `${progressPercent}%` }" />
          </div>
        </template>
        <template v-else>
          <div class="review-progress-numbers">
            <strong>完成</strong>
          </div>
          <div class="review-progress-track">
            <span class="review-progress-fill" style="width: 100%" />
          </div>
        </template>
      </div>
    </header>

    <main class="review-shell" :class="{ 'review-shell-with-rating': revealed && !done && current }">
      <section v-if="done" class="review-done-card">
        <div class="review-done-icon">
          <CheckCircle class="h-14 w-14 text-[#2f7cff]" />
        </div>
        <h2 class="review-done-title">本轮错题复习完成</h2>
        <p class="review-done-text">这次一共完成 {{ items.length }} 道题。回到错题本继续整理一两道，记忆会更稳。</p>
        <button class="review-primary-button mt-5" type="button" @click="router.push({ name: 'ErrorNotebook' })">返回错题本</button>
      </section>

      <template v-else-if="current">
        <section class="review-hero-card">
          <div class="review-hero-glow" aria-hidden="true" />
          <div class="review-hero-grid" :class="{ 'review-hero-grid-no-image': !imageSrc }">
            <div v-if="imageSrc" class="review-hero-media">
              <img :src="imageSrc" class="review-thumb" @error="handleImageError" />
            </div>

            <div class="review-hero-content">
              <div class="review-hero-pills">
                <div class="review-mini-pill">
                  <Clock3 class="h-4 w-4" />
                  今日待复习
                </div>
                <div class="review-mini-pill review-mini-pill-level">
                  <Target class="h-4 w-4" />
                  Lv. {{ current.masteryLevel }}
                </div>
              </div>

              <RichText
                class="review-question-text"
                :text="current.questionText"
                fallback="这道题还没有题干，先根据图片回忆核心条件。"
              />

              <div v-if="knowledgePoints.length" class="review-question-tags-row">
                <div
                  ref="knowledgeTagsRef"
                  class="review-question-tags"
                  :class="{ 'review-question-tags-collapsed': !knowledgeExpanded }"
                >
                  <span v-for="point in knowledgePoints" :key="point" class="review-chip">
                    {{ point }}
                  </span>
                </div>
                <button
                  v-if="knowledgeOverflowing"
                  class="review-inline-expand"
                  type="button"
                  @click="knowledgeExpanded = !knowledgeExpanded"
                >
                  <span>{{ knowledgeExpanded ? '收起' : '更多' }}</span>
                  <ChevronDown class="h-4 w-4" :class="{ 'review-inline-expand-open': knowledgeExpanded }" />
                </button>
              </div>

              <button v-if="!revealed" class="review-expand-answer" type="button" @click="revealed = true">
                <span>展开答案</span>
                <ChevronDown class="h-5 w-5" />
              </button>
            </div>

            <div class="review-hero-ornament" aria-hidden="true">
              <span class="review-cube review-cube-a" />
              <span class="review-cube review-cube-b" />
              <span class="review-cube review-cube-c" />
              <span class="review-cube review-cube-d" />
              <span class="review-cube review-cube-e" />
            </div>
          </div>
        </section>

        <section v-if="revealed" class="review-answer-card">
          <div class="review-tabs">
            <button class="review-tab" :class="{ 'review-tab-active': activeTab === 'answer' }" type="button" @click="activeTab = 'answer'">
              答案与解析
            </button>
            <button class="review-tab" :class="{ 'review-tab-active': activeTab === 'mistake' }" type="button" @click="activeTab = 'mistake'">
              错因与笔记
            </button>
          </div>

          <div class="review-tab-panel">
            <article class="review-content-card">
              <template v-if="activeTab === 'answer'">
                <section class="review-content-section">
                  <div class="review-card-head">
                    <CheckCircle class="h-5 w-5 text-[#2f7cff]" />
                    <h2>标准答案</h2>
                  </div>
                  <RichText class="review-content-text" :text="current.answerText" fallback="暂无答案" />
                </section>

                <section class="review-content-section review-content-section-divider">
                  <div class="review-card-head">
                    <Lightbulb class="h-5 w-5 text-[#2f7cff]" />
                    <h2>解析</h2>
                  </div>
                  <RichText class="review-content-text" :text="current.analysis" fallback="暂无解析" />
                </section>
              </template>

              <template v-else>
                <template v-if="hasMistakeContent">
                  <section v-if="current.wrongAnswerText" class="review-content-section">
                    <div class="review-card-head">
                      <Brain class="h-5 w-5 text-[#ff786d]" />
                      <h2>错答记录</h2>
                    </div>
                    <RichText class="review-content-text" :text="current.wrongAnswerText" />
                  </section>

                  <section
                    v-if="current.mistakeAnalysis"
                    class="review-content-section"
                    :class="{ 'review-content-section-divider': current.wrongAnswerText }"
                  >
                    <div class="review-card-head">
                      <Brain class="h-5 w-5 text-[#ff786d]" />
                      <h2>错因分析</h2>
                    </div>
                    <RichText class="review-content-text" :text="current.mistakeAnalysis" />
                  </section>

                  <section
                    v-if="current.userNotes"
                    class="review-content-section"
                    :class="{ 'review-content-section-divider': current.wrongAnswerText || current.mistakeAnalysis }"
                  >
                    <div class="review-card-head">
                      <NotebookPen class="h-5 w-5 text-[#2f7cff]" />
                      <h2>笔记</h2>
                    </div>
                    <RichText class="review-content-text" :text="current.userNotes" />
                  </section>
                </template>

                <section v-else class="review-empty-state">
                  <div class="review-card-head">
                    <Brain class="h-5 w-5 text-[#ff786d]" />
                    <h2>错因与笔记</h2>
                  </div>
                  <p>这道题还没有记录错因，复习后可以补充。</p>
                </section>
              </template>
            </article>
          </div>
        </section>

        <button v-if="revealed" class="review-note-preview" type="button" @click="activeTab = 'mistake'">
          <span class="review-note-icon">
            <Lightbulb class="h-5 w-5" />
          </span>
          <span class="review-note-copy">
            <strong>错因与笔记（预览）</strong>
            <span v-if="hasMistakeContent">查看错答、错因分析和复习笔记</span>
            <span v-else>复习后可补充：这题为什么错、下次怎么判断</span>
          </span>
          <ChevronDown class="h-5 w-5 rotate-[-90deg]" />
        </button>
      </template>
    </main>

    <footer v-if="revealed && !done && current" class="review-rating-bar">
      <div class="review-rating-shell">
        <button class="review-score review-score-red" type="button" @click="rate(0)">
          <span class="review-face review-face-sad" aria-hidden="true">
            <span class="review-face-eyes">··</span>
            <span class="review-face-mouth" />
          </span>
          <span class="review-score-copy">
            <span class="review-score-title">忘了</span>
            <span class="review-score-hint">基本没想起来</span>
          </span>
        </button>
        <button class="review-score review-score-warm" type="button" @click="rate(3)">
          <span class="review-face review-face-flat" aria-hidden="true">
            <span class="review-face-eyes">··</span>
            <span class="review-face-mouth" />
          </span>
          <span class="review-score-copy">
            <span class="review-score-title">模糊</span>
            <span class="review-score-hint">思路还不稳定</span>
          </span>
        </button>
        <button class="review-score review-score-green" type="button" @click="rate(5)">
          <span class="review-face review-face-smile" aria-hidden="true">
            <span class="review-face-eyes">··</span>
            <span class="review-face-mouth" />
          </span>
          <span class="review-score-copy">
            <span class="review-score-title">掌握</span>
            <span class="review-score-hint">能独立做出</span>
          </span>
        </button>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.review-page {
  background:
    radial-gradient(circle at 18% 0%, rgba(98, 146, 255, 0.16), transparent 24%),
    radial-gradient(circle at 86% 8%, rgba(171, 204, 255, 0.2), transparent 22%),
    linear-gradient(180deg, #fffefe 0%, #f8fbff 34%, #edf4ff 100%);
}

.review-topbar {
  display: grid;
  grid-template-columns: 3.35rem minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.82rem;
  padding: 0.95rem 1.15rem 0;
}

.review-nav-button {
  display: grid;
  width: 3rem;
  height: 3rem;
  place-items: center;
  border: 1px solid rgba(209, 221, 247, 0.94);
  border-radius: 1.15rem;
  background: rgba(255, 255, 255, 0.96);
  color: #1e3f8f;
  box-shadow:
    0 14px 30px rgba(78, 113, 188, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.96);
}

.review-title-block {
  min-width: 0;
  text-align: center;
}

.review-title {
  margin: 0;
  color: #132863;
  font-size: 1.36rem;
  line-height: 1.1;
  font-weight: 950;
  letter-spacing: 0.02em;
}

.review-subtitle {
  margin: 0.34rem auto 0;
  max-width: 16rem;
  color: #6c82af;
  font-size: 0.84rem;
  line-height: 1.42;
  font-weight: 650;
}

.review-progress-pill {
  min-width: 6.15rem;
  padding: 0.7rem 0.9rem 0.62rem;
  border: 1px solid rgba(212, 224, 248, 0.94);
  border-radius: 1.45rem;
  background: rgba(255, 255, 255, 0.96);
  box-shadow:
    0 16px 30px rgba(80, 110, 176, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.review-progress-pill-complete {
  min-width: 5.7rem;
}

.review-progress-numbers {
  display: flex;
  align-items: baseline;
  justify-content: center;
  gap: 0.25rem;
  color: #6f82aa;
  font-weight: 700;
}

.review-progress-numbers strong {
  color: #1f66ff;
  font-size: 1.58rem;
  line-height: 1;
  font-weight: 950;
}

.review-progress-numbers span {
  font-size: 0.92rem;
}

.review-progress-track {
  margin-top: 0.48rem;
  height: 0.34rem;
  border-radius: 999px;
  background: rgba(221, 229, 243, 0.86);
  overflow: hidden;
}

.review-progress-fill {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, #1f66ff 0%, #4d8dff 100%);
}

.review-shell {
  padding: 1.1rem 1rem 2rem;
}

.review-shell-with-rating {
  padding-bottom: calc(9.75rem + var(--safe-area-bottom));
}

.review-hero-card,
.review-answer-card,
.review-done-card {
  position: relative;
  overflow: hidden;
  border: 1px solid rgba(197, 216, 248, 0.96);
  border-radius: 2rem;
  background:
    radial-gradient(circle at 82% 18%, rgba(47, 124, 255, 0.12), transparent 30%),
    linear-gradient(135deg, rgba(255, 255, 255, 0.98), rgba(238, 245, 255, 0.96));
  box-shadow:
    0 20px 44px rgba(93, 123, 194, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.review-hero-card {
  padding: 1.35rem 1.25rem;
}

.review-hero-glow {
  position: absolute;
  inset: auto auto -4rem -3rem;
  width: 11rem;
  height: 11rem;
  border-radius: 999px;
  background: radial-gradient(circle, rgba(103, 153, 255, 0.12), transparent 70%);
  pointer-events: none;
}

.review-hero-grid {
  position: relative;
  z-index: 1;
  display: grid;
  grid-template-columns: 9rem minmax(0, 1fr);
  gap: 1.15rem;
  align-items: start;
}

.review-hero-grid-no-image {
  grid-template-columns: minmax(0, 1fr);
}

.review-hero-media {
  position: relative;
  z-index: 2;
}

.review-thumb {
  width: 100%;
  height: 11.4rem;
  object-fit: cover;
  border-radius: 1.28rem;
  border: 1px solid rgba(196, 214, 248, 0.96);
  background: rgba(255, 255, 255, 0.94);
  box-shadow:
    0 12px 28px rgba(95, 122, 181, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.review-hero-content {
  position: relative;
  z-index: 2;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  padding-right: 7.2rem;
}

.review-hero-grid-no-image .review-hero-content {
  padding-right: 8.6rem;
}

.review-hero-pills {
  display: flex;
  flex-wrap: wrap;
  gap: 0.55rem;
  margin-bottom: 0.82rem;
}

.review-mini-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.38rem;
  min-height: 2rem;
  padding: 0 0.84rem;
  border: 1px solid rgba(189, 210, 248, 0.94);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.88);
  color: #2b62d8;
  font-size: 0.83rem;
  font-weight: 850;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.review-mini-pill-level {
  color: #3f69d5;
}

.review-question-text {
  margin: 0;
  color: #152b63;
  font-size: 1.12rem;
  line-height: 1.72;
  font-weight: 850;
  white-space: pre-wrap;
}

.review-question-tags-row {
  display: flex;
  align-items: flex-start;
  gap: 0.45rem;
  margin-top: 1rem;
  width: 100%;
}

.review-question-tags {
  flex: 1;
  display: flex;
  flex-wrap: wrap;
  gap: 0.42rem;
  min-width: 0;
}

.review-question-tags-collapsed {
  max-height: 1.8rem;
  overflow: hidden;
}

.review-chip {
  display: inline-flex;
  align-items: center;
  min-height: 1.8rem;
  padding: 0 0.7rem;
  border: 1px solid rgba(191, 213, 250, 0.96);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.84);
  color: #2670ff;
  font-size: 0.78rem;
  font-weight: 800;
}

.review-inline-expand {
  display: inline-flex;
  align-items: center;
  gap: 0.24rem;
  min-height: 1.8rem;
  padding: 0 0.72rem;
  border: 1px solid rgba(199, 216, 248, 0.96);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.86);
  color: #3c6fdb;
  font-size: 0.74rem;
  font-weight: 800;
}

.review-inline-expand-open {
  transform: rotate(180deg);
}

.review-expand-answer,
.review-primary-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.48rem;
  min-height: 3.9rem;
  border: 0;
  border-radius: 1.4rem;
  padding: 0 1.7rem;
  background: linear-gradient(135deg, #2f7cff 0%, #1f66ff 100%);
  color: white;
  font-size: 1rem;
  font-weight: 900;
  box-shadow:
    0 16px 32px rgba(34, 102, 255, 0.26),
    inset 0 1px 0 rgba(255, 255, 255, 0.28);
}

.review-expand-answer {
  margin-top: 1.2rem;
}

.review-hero-ornament {
  position: absolute;
  right: 0.35rem;
  bottom: 0.1rem;
  width: 9.5rem;
  height: 10rem;
  pointer-events: none;
  opacity: 0.82;
}

.review-cube {
  position: absolute;
  border: 1px solid rgba(179, 205, 255, 0.56);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.52), rgba(214, 231, 255, 0.18));
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.72);
}

.review-cube::before,
.review-cube::after {
  content: "";
  position: absolute;
  border: 1px solid rgba(179, 205, 255, 0.46);
}

.review-cube::before {
  inset: -0.42rem 0.42rem auto auto;
  width: 0.92rem;
  height: 0.92rem;
  transform: skewY(-30deg);
  border-left: 0;
  border-bottom: 0;
}

.review-cube::after {
  inset: auto -0.42rem 0.42rem auto;
  width: 0.92rem;
  height: calc(100% - 0.1rem);
  transform: skewY(30deg);
  border-left: 0;
  border-top: 0;
}

.review-cube-a {
  right: 1.2rem;
  top: 0.8rem;
  width: 2rem;
  height: 2rem;
}

.review-cube-b {
  right: 2.9rem;
  top: 2.8rem;
  width: 2.25rem;
  height: 2.25rem;
}

.review-cube-c {
  right: 0.85rem;
  top: 3.2rem;
  width: 2.15rem;
  height: 2.15rem;
}

.review-cube-d {
  right: 4.7rem;
  top: 4.5rem;
  width: 1.7rem;
  height: 1.7rem;
}

.review-cube-e {
  right: 3.3rem;
  bottom: 0.75rem;
  width: 1.95rem;
  height: 1.95rem;
}

.review-answer-card {
  margin-top: 1.1rem;
  border-radius: 2rem;
  background:
    radial-gradient(circle at 20% 0%, rgba(255, 255, 255, 0.98), transparent 35%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.98) 0%, rgba(245, 249, 255, 0.98) 100%);
  box-shadow:
    0 18px 42px rgba(97, 126, 190, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.review-tabs {
  display: flex;
  gap: 0.28rem;
  padding: 0.9rem 0.9rem 0;
}

.review-tab {
  position: relative;
  flex: 1;
  min-height: 3.45rem;
  border: 0;
  border-radius: 1.3rem 1.3rem 0 0;
  background: linear-gradient(180deg, rgba(246, 249, 255, 0.96) 0%, rgba(239, 245, 255, 0.96) 100%);
  color: #8091b6;
  font-size: 1rem;
  font-weight: 900;
}

.review-tab-active {
  background: rgba(255, 255, 255, 0.98);
  color: #1f66ff;
}

.review-tab-active::after {
  content: "";
  position: absolute;
  left: 50%;
  bottom: 0.4rem;
  width: 3.4rem;
  height: 0.22rem;
  border-radius: 999px;
  background: linear-gradient(90deg, #1f66ff 0%, #4d8dff 100%);
  transform: translateX(-50%);
}

.review-tab-panel {
  padding: 0 0.9rem 0.95rem;
}

.review-content-card {
  border: 1px solid rgba(224, 233, 247, 0.96);
  border-radius: 1.55rem;
  background: rgba(255, 255, 255, 0.98);
  padding: 1.15rem;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.review-content-section + .review-content-section {
  margin-top: 1rem;
}

.review-content-section-divider {
  padding-top: 1rem;
  border-top: 1px dashed rgba(222, 231, 245, 0.96);
}

.review-card-head {
  display: flex;
  align-items: center;
  gap: 0.58rem;
  margin-bottom: 0.62rem;
}

.review-card-head h2 {
  margin: 0;
  font-size: 1rem;
  font-weight: 900;
  color: #17306b;
}

.review-content-text {
  margin: 0;
  white-space: pre-wrap;
  color: #1b315f;
  font-size: 0.97rem;
  line-height: 1.8;
}

.review-empty-state {
  padding: 0.25rem 0;
}

.review-empty-state p {
  margin: 0;
  color: #6e80a7;
  font-size: 0.95rem;
  line-height: 1.72;
}

.review-done-card {
  padding: 1.8rem 1.4rem;
  text-align: center;
}

.review-done-icon {
  display: flex;
  justify-content: center;
  margin-bottom: 1rem;
}

.review-done-title {
  margin: 0;
  color: #1d2e62;
  font-size: 1.42rem;
  font-weight: 900;
}

.review-done-text {
  margin: 0.6rem auto 0;
  max-width: 20rem;
  color: #6d7ea9;
  font-size: 0.95rem;
  line-height: 1.7;
}

.review-rating-bar {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 30;
  padding: 0.9rem 0.95rem calc(var(--safe-area-bottom) + 0.92rem);
  background: linear-gradient(180deg, rgba(244, 248, 255, 0) 0%, rgba(244, 248, 255, 0.84) 28%, rgba(244, 248, 255, 0.95) 100%);
}

.review-rating-shell {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.72rem;
  padding: 0.72rem;
  border: 1px solid rgba(215, 226, 245, 0.96);
  border-radius: 1.7rem;
  background: rgba(255, 255, 255, 0.82);
  backdrop-filter: blur(18px);
  box-shadow:
    0 18px 36px rgba(79, 107, 172, 0.12),
    inset 0 1px 0 rgba(255, 255, 255, 0.96);
}

.review-score {
  display: flex;
  min-height: 5rem;
  align-items: center;
  gap: 0.72rem;
  border: 1px solid transparent;
  border-radius: 1.4rem;
  padding: 0.95rem 0.88rem;
  text-align: left;
  box-shadow:
    0 10px 22px rgba(95, 119, 173, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.96);
}

.review-score-red {
  border-color: rgba(255, 194, 194, 0.98);
  background: linear-gradient(180deg, rgba(255, 251, 251, 0.98), rgba(255, 244, 244, 0.98));
  color: #ef4e53;
}

.review-score-warm {
  border-color: rgba(255, 216, 171, 0.98);
  background: linear-gradient(180deg, rgba(255, 252, 246, 0.98), rgba(255, 245, 229, 0.98));
  color: #f29118;
}

.review-score-green {
  border-color: rgba(185, 237, 197, 0.98);
  background: linear-gradient(180deg, rgba(248, 255, 250, 0.98), rgba(238, 250, 242, 0.98));
  color: #18a957;
}

.review-score-icon {
  display: inline-grid;
  width: 2.2rem;
  height: 2.2rem;
  place-items: center;
  flex-shrink: 0;
  border-radius: 999px;
  border: 2px solid currentColor;
  font-size: 1rem;
  font-weight: 900;
  line-height: 1;
}

.review-score-copy {
  display: flex;
  min-width: 0;
  flex-direction: column;
  align-items: flex-start;
}

.review-score-title {
  font-size: 1rem;
  font-weight: 900;
}

.review-score-hint {
  margin-top: 0.2rem;
  color: rgba(41, 55, 90, 0.68);
  font-size: 0.76rem;
  line-height: 1.35;
}

@media (max-width: 720px) {
  .review-topbar {
    grid-template-columns: 3rem minmax(0, 1fr);
    gap: 0.7rem;
    padding-right: 1rem;
    padding-left: 1rem;
  }

  .review-nav-button {
    width: 2.8rem;
    height: 2.8rem;
    border-radius: 1.05rem;
  }

  .review-title {
    font-size: 1.22rem;
  }

  .review-subtitle {
    font-size: 0.8rem;
  }

  .review-progress-pill {
    grid-column: 2;
    justify-self: end;
    min-width: 5.5rem;
    margin-top: 0.15rem;
    padding: 0.58rem 0.8rem 0.52rem;
  }

  .review-progress-numbers strong {
    font-size: 1.38rem;
  }

  .review-shell {
    padding-top: 1rem;
    padding-right: 1rem;
    padding-left: 1rem;
  }

  .review-hero-card {
    padding: 1.15rem 1rem 1rem;
  }

  .review-hero-grid {
    grid-template-columns: 5.9rem minmax(0, 1fr);
    gap: 0.95rem;
  }

  .review-thumb {
    height: 8.2rem;
    border-radius: 1.05rem;
  }

  .review-hero-content,
  .review-hero-grid-no-image .review-hero-content {
    padding-right: 0;
  }

  .review-question-text {
    font-size: 0.98rem;
    line-height: 1.64;
  }

  .review-hero-ornament {
    right: -0.3rem;
    bottom: -0.1rem;
    width: 7rem;
    height: 7rem;
    opacity: 0.5;
  }

  .review-expand-answer,
  .review-primary-button {
    min-height: 3.55rem;
    width: 100%;
    font-size: 0.96rem;
  }

  .review-tabs {
    padding: 0.72rem 0.72rem 0;
  }

  .review-tab {
    min-height: 3rem;
    font-size: 0.9rem;
  }

  .review-tab-panel {
    padding: 0 0.72rem 0.72rem;
  }

  .review-content-card {
    padding: 1rem;
  }

  .review-rating-bar {
    padding-right: 0.8rem;
    padding-left: 0.8rem;
  }

  .review-rating-shell {
    gap: 0.6rem;
    padding: 0.62rem;
  }

  .review-score {
    min-height: 4.7rem;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.52rem;
    padding: 0.82rem 0.74rem;
  }

  .review-score-icon {
    width: 2rem;
    height: 2rem;
  }

  .review-score-title {
    font-size: 0.94rem;
  }

  .review-score-hint {
    font-size: 0.72rem;
  }
}

/* Strong visual alignment with the approved UI mock: keep the header single-row on phones,
   make the hero card more compact and make the bottom rating area feel like a premium floating bar. */
.review-topbar {
  max-width: 46rem;
  margin: 0 auto;
  grid-template-columns: 3.05rem minmax(0, 1fr) 5.85rem;
}

.review-progress-pill {
  grid-column: auto;
  justify-self: end;
  margin-top: 0;
}

.review-hero-card,
.review-answer-card,
.review-note-preview,
.review-done-card {
  max-width: 46rem;
  margin-inline: auto;
}

.review-hero-card {
  border-radius: 1.82rem;
  padding: 1.18rem 1.08rem 1.08rem;
}

.review-hero-pills {
  justify-content: flex-start;
}

.review-hero-grid {
  display: grid;
  grid-template-columns: 9.25rem minmax(0, 1fr);
  gap: 1.08rem;
  align-items: start;
}

.review-thumb {
  height: 9.25rem;
  border-radius: 1.18rem;
}

.review-hero-content {
  padding-right: 5.2rem;
}

.review-hero-grid-no-image .review-hero-content {
  max-width: 32rem;
  padding-right: 6rem;
}

.review-question-text {
  max-width: 28.5rem;
  font-size: 1.04rem;
  line-height: 1.72;
}

.review-question-tags-row {
  max-width: 28.5rem;
}

.review-expand-answer {
  align-self: flex-start;
  min-width: 17rem;
  padding-right: 1.5rem;
  padding-left: 1.5rem;
}

.review-hero-ornament {
  right: 0.5rem;
  bottom: 0.55rem;
  width: 7.25rem;
  height: 7.6rem;
  opacity: 0.54;
}

.review-cube-a {
  right: 0.95rem;
  top: 0.45rem;
  width: 1.7rem;
  height: 1.7rem;
}

.review-cube-b {
  right: 2.35rem;
  top: 2.15rem;
  width: 1.9rem;
  height: 1.9rem;
}

.review-cube-c {
  right: 0.65rem;
  top: 2.45rem;
  width: 1.82rem;
  height: 1.82rem;
}

.review-cube-d {
  right: 3.9rem;
  top: 3.85rem;
  width: 1.38rem;
  height: 1.38rem;
}

.review-cube-e {
  right: 2.7rem;
  bottom: 0.25rem;
  width: 1.62rem;
  height: 1.62rem;
}

.review-answer-card {
  padding: 0.78rem;
}

.review-content-card {
  border-radius: 0 0 1.45rem 1.45rem;
  padding: 1.18rem 1.05rem;
}

.review-note-preview {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.74rem;
  width: 100%;
  margin-top: 0.98rem;
  padding: 1rem 1.04rem;
  text-align: left;
  background: rgba(255, 255, 255, 0.94);
}

.review-note-icon {
  display: grid;
  width: 2.3rem;
  height: 2.3rem;
  place-items: center;
  border-radius: 0.82rem;
  background: rgba(232, 240, 255, 0.94);
  color: #1f66ff;
}

.review-note-copy {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 0.28rem;
}

.review-note-copy strong {
  color: #1f66ff;
  font-size: 1rem;
  font-weight: 950;
}

.review-note-copy span {
  color: #5e6f99;
  font-size: 0.84rem;
  line-height: 1.42;
}

.review-face {
  position: relative;
  display: grid;
  width: 2.35rem;
  height: 2.35rem;
  place-items: center;
  flex-shrink: 0;
  border: 2px solid currentColor;
  border-radius: 999px;
}

.review-face-eyes {
  position: absolute;
  top: 0.48rem;
  left: 50%;
  transform: translateX(-50%);
  font-size: 1.18rem;
  letter-spacing: 0.1rem;
  line-height: 0.8;
}

.review-face-mouth {
  position: absolute;
  left: 50%;
  bottom: 0.56rem;
  width: 0.92rem;
  height: 0.34rem;
  transform: translateX(-50%);
}

.review-face-sad .review-face-mouth {
  border-top: 2px solid currentColor;
  border-radius: 999px 999px 0 0;
}

.review-face-flat .review-face-mouth {
  height: 0;
  border-top: 2px solid currentColor;
}

.review-face-smile .review-face-mouth {
  border-bottom: 2px solid currentColor;
  border-radius: 0 0 999px 999px;
}

.review-rating-shell {
  gap: 0.58rem;
  padding: 0.64rem;
  border-radius: 1.55rem;
}

.review-score {
  min-height: 5rem;
  flex-direction: row;
  align-items: center;
  gap: 0.62rem;
  border-radius: 1.28rem;
  padding: 0.82rem 0.68rem;
}

@media (max-width: 420px) {
  .review-topbar {
    grid-template-columns: 2.86rem minmax(0, 1fr) 5.45rem;
    gap: 0.46rem;
    padding-right: 0.78rem;
    padding-left: 0.78rem;
  }

  .review-nav-button {
    width: 2.78rem;
    height: 2.78rem;
    border-radius: 1rem;
  }

  .review-title {
    font-size: 1.25rem;
  }

  .review-subtitle {
    font-size: 0.72rem;
    transform: scale(0.94);
    transform-origin: center;
  }

  .review-progress-pill {
    width: 5.45rem;
    min-width: 0;
    padding-right: 0.68rem;
    padding-left: 0.68rem;
  }

  .review-hero-grid {
    grid-template-columns: 6.35rem minmax(0, 1fr);
    gap: 0.74rem;
  }

  .review-thumb {
    height: 6.35rem;
  }

  .review-question-text {
    font-size: 0.98rem;
    line-height: 1.58;
  }

  .review-score {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.44rem;
    padding: 0.72rem 0.58rem;
  }

  .review-face {
    width: 2.05rem;
    height: 2.05rem;
  }
}

@media (min-width: 721px) {
  .review-shell {
    max-width: 48rem;
    margin: 0 auto;
    padding-right: 1.5rem;
    padding-left: 1.5rem;
  }

  .review-rating-bar {
    left: 50%;
    right: auto;
    width: min(calc(100vw - 2rem), 50rem);
    transform: translateX(-50%);
  }

  .review-rating-shell {
    margin: 0 auto;
    max-width: 46rem;
  }

  .review-expand-answer {
    min-width: 18.5rem;
  }
}

@media (min-width: 1100px) {
  .review-shell {
    max-width: 44rem;
  }

  .review-topbar,
  .review-hero-card,
  .review-answer-card,
  .review-note-preview,
  .review-done-card {
    max-width: 44rem;
  }

  .review-rating-bar {
    width: min(calc(100vw - 2rem), 46rem);
  }
}
</style>
