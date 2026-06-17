<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'
import {
  ArrowLeft,
  Brain,
  CheckCircle,
  ChevronDown,
  ChevronUp,
  Clock3,
  Ellipsis,
  Lightbulb,
  NotebookPen,
  RotateCcw,
  Sparkles,
  Target,
} from 'lucide-vue-next'
import * as errorApi from '../api/errorItem'

const router = useRouter()
const items = ref<errorApi.ErrorItem[]>([])
const index = ref(0)
const revealed = ref(false)
const startedAt = ref(Date.now())
const done = ref(false)
const activeTab = ref<'answer' | 'mistake'>('answer')
const knowledgeExpanded = ref(false)
const knowledgeOverflowing = ref(false)
const knowledgeTagsRef = ref<HTMLElement | null>(null)

const current = computed(() => items.value[index.value] || null)
const imageSrc = computed(() => current.value?.localImagePath ? convertFileSrc(current.value.localImagePath) : current.value?.remoteImageUrl || '')
const progressLabel = computed(() => done.value ? '完成' : `${index.value + 1} / ${items.value.length}`)
const knowledgePoints = computed(() => current.value ? errorApi.parseKnowledgePoints(current.value.knowledgePoints) : [])

onMounted(async () => {
  items.value = await errorApi.getDueErrorItems()
  if (items.value.length === 0) done.value = true
  await nextTick()
  updateKnowledgeOverflow()
})

watch(
  () => [current.value?.id, knowledgePoints.value.join('|')],
  async () => {
    await nextTick()
    updateKnowledgeOverflow()
  },
)

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

  const chips = Array.from(el.querySelectorAll<HTMLElement>('.error-chip'))
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
    <header class="error-header header-safe-top">
      <button class="error-header-button" type="button" @click="router.push({ name: 'ErrorNotebook' })">
        <ArrowLeft class="h-6 w-6" />
      </button>
      <div class="text-center">
        <h1 class="error-header-title">错题复习</h1>
        <p class="error-header-subtitle">{{ progressLabel }}</p>
      </div>
      <button class="error-header-button" type="button" aria-label="more">
        <Ellipsis class="h-6 w-6" />
      </button>
    </header>

    <main class="error-main">
      <section v-if="done" class="error-hero-card error-done-card">
        <CheckCircle class="mb-4 h-14 w-14 text-[#2f7cff]" />
        <h2 class="error-done-title">本轮错题复习完成</h2>
        <p class="error-done-text">这次一共完成 {{ items.length }} 道题。回到错题本继续整理一两道，记忆会更稳。</p>
        <button class="error-primary-button mt-5" type="button" @click="router.push({ name: 'ErrorNotebook' })">返回错题本</button>
      </section>

      <template v-else-if="current">
        <section class="error-hero-card">
          <div class="error-hero-top">
            <img v-if="imageSrc" :src="imageSrc" class="error-thumb" />

            <div class="error-hero-content">
              <div class="error-mini-stats">
                <div class="error-mini-pill">
                  <Clock3 class="h-4 w-4" />
                  {{ current.nextReview }}
                </div>
                <div class="error-mini-pill">
                  <Target class="h-4 w-4" />
                  Lv. {{ current.masteryLevel }}
                </div>
              </div>

              <p class="error-question-text">{{ current.questionText || '这道题还没有题干，先根据图片回忆核心条件。' }}</p>

              <div v-if="knowledgePoints.length" class="error-question-tags-row">
                <div
                  ref="knowledgeTagsRef"
                  class="error-question-tags"
                  :class="{ 'error-question-tags-collapsed': !knowledgeExpanded }"
                >
                  <span v-for="point in knowledgePoints" :key="point" class="error-chip">
                    {{ point }}
                  </span>
                </div>
                <button
                  v-if="knowledgeOverflowing"
                  class="error-inline-expand"
                  type="button"
                  @click="knowledgeExpanded = !knowledgeExpanded"
                >
                  <ChevronDown v-if="!knowledgeExpanded" class="h-4.5 w-4.5" />
                  <ChevronUp v-else class="h-4.5 w-4.5" />
                </button>
              </div>
            </div>
          </div>
        </section>

        <section v-if="revealed" class="error-tabs-panel">
          <div class="error-tabs">
            <button class="error-tab" :class="{ 'error-tab-active': activeTab === 'answer' }" type="button" @click="activeTab = 'answer'">
              答案与解析
            </button>
            <button class="error-tab" :class="{ 'error-tab-active': activeTab === 'mistake' }" type="button" @click="activeTab = 'mistake'">
              错因与笔记
            </button>
          </div>

          <div class="error-tab-content">
            <article class="error-content-card">
              <template v-if="activeTab === 'answer'">
                <section class="error-content-section">
                  <div class="error-card-head">
                    <Sparkles class="h-5 w-5 text-[#2f7cff]" />
                    <h2>标准答案</h2>
                  </div>
                  <p class="error-content-text">{{ current.answerText || '暂无答案' }}</p>
                </section>

                <section class="error-content-section error-content-section-divider">
                  <div class="error-card-head">
                    <Lightbulb class="h-5 w-5 text-[#2f7cff]" />
                    <h2>解析</h2>
                  </div>
                  <p class="error-content-text">{{ current.analysis || '暂无解析' }}</p>
                </section>
              </template>

              <template v-else>
                <section v-if="current.wrongAnswerText" class="error-content-section">
                  <div class="error-card-head">
                    <Brain class="h-5 w-5 text-[#ff786d]" />
                    <h2>错答记录</h2>
                  </div>
                  <p class="error-content-text">{{ current.wrongAnswerText }}</p>
                </section>

                <section v-if="current.mistakeAnalysis" class="error-content-section" :class="{ 'error-content-section-divider': current.wrongAnswerText }">
                  <div class="error-card-head">
                    <Brain class="h-5 w-5 text-[#ff786d]" />
                    <h2>错因分析</h2>
                  </div>
                  <p class="error-content-text">{{ current.mistakeAnalysis }}</p>
                </section>

                <section v-if="current.userNotes" class="error-content-section" :class="{ 'error-content-section-divider': current.wrongAnswerText || current.mistakeAnalysis }">
                  <div class="error-card-head">
                    <NotebookPen class="h-5 w-5 text-[#2f7cff]" />
                    <h2>笔记</h2>
                  </div>
                  <p class="error-content-text">{{ current.userNotes }}</p>
                </section>
              </template>
            </article>
          </div>
        </section>
      </template>
    </main>

    <footer v-if="!done && current" class="error-actions" :class="{ 'error-actions-rating': revealed }">
      <template v-if="!revealed">
        <button class="error-action error-action-ghost" type="button" @click="router.push({ name: 'ErrorNotebook' })">
          返回错题本
        </button>
        <button class="error-action error-action-primary error-action-wide" type="button" @click="revealed = true">
          <RotateCcw class="h-5 w-5" />
          我想好了，展开答案
        </button>
      </template>

      <template v-else>
        <button class="error-score error-score-red" type="button" @click="rate(0)">
          <span class="error-score-title">忘了</span>
          <span class="error-score-hint">基本没想起来</span>
        </button>
        <button class="error-score error-score-warm" type="button" @click="rate(3)">
          <span class="error-score-title">模糊</span>
          <span class="error-score-hint">能做一半，还不稳</span>
        </button>
        <button class="error-score error-score-green" type="button" @click="rate(5)">
          <span class="error-score-title">掌握</span>
          <span class="error-score-hint">思路清楚，能独立做</span>
        </button>
      </template>
    </footer>
  </div>
</template>

<style scoped>
.review-page {
  background:
    radial-gradient(circle at top center, rgba(146, 187, 255, 0.18), transparent 28%),
    linear-gradient(180deg, #fbfdff 0%, #f3f7ff 30%, #edf3ff 100%);
}

.error-header {
  display: grid;
  grid-template-columns: 2.75rem 1fr 2.75rem;
  align-items: center;
  gap: 0.75rem;
  padding: 0.9rem 1.5rem 0;
}

.error-header-title {
  margin: 0;
  text-align: center;
  font-size: 1.1rem;
  font-weight: 900;
  color: #18274f;
}

.error-header-subtitle {
  margin: 0.2rem 0 0;
  text-align: center;
  font-size: 0.76rem;
  color: #8ea0c6;
}

.error-header-button {
  display: grid;
  width: 2.75rem;
  height: 2.75rem;
  place-items: center;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: #23345f;
}

.error-main {
  padding: 1.15rem 1.5rem 8.6rem;
}

.error-hero-card,
.error-tabs-panel {
  overflow: hidden;
  border: 1px solid rgba(214, 227, 255, 0.95);
  border-radius: 2rem;
  background:
    radial-gradient(circle at top right, rgba(255, 255, 255, 0.48), transparent 30%),
    linear-gradient(180deg, #eef5ff 0%, #eaf2ff 100%);
  box-shadow:
    0 20px 44px rgba(93, 123, 194, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.96);
}

.error-hero-card {
  padding: 1.35rem;
}

.error-hero-top {
  display: grid;
  grid-template-columns: 9.4rem minmax(0, 1fr);
  gap: 1.15rem;
}

.error-thumb {
  width: 100%;
  height: 11.2rem;
  object-fit: cover;
  border-radius: 1.15rem;
  border: 3px solid rgba(255, 255, 255, 0.96);
  box-shadow: 0 14px 28px rgba(106, 127, 176, 0.12);
}

.error-hero-content {
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.error-mini-stats {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
  margin-bottom: 0.45rem;
}

.error-mini-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  min-height: 1.95rem;
  padding: 0 0.72rem;
  border: 1px solid rgba(188, 212, 255, 0.9);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.82);
  color: #5182de;
  font-size: 0.76rem;
  font-weight: 800;
}

.error-question-text {
  margin: 0;
  color: #18284f;
  font-size: 1rem;
  line-height: 1.78;
  white-space: pre-wrap;
}

.error-question-tags-row {
  display: flex;
  align-items: flex-start;
  gap: 0.35rem;
  margin-top: 0.08rem;
}

.error-question-tags {
  flex: 1;
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  min-width: 0;
}

.error-question-tags-collapsed {
  max-height: 1.45rem;
  overflow: hidden;
}

.error-chip {
  display: inline-flex;
  align-items: center;
  min-height: 1.45rem;
  padding: 0 0.5rem;
  border: 1px solid rgba(188, 212, 255, 0.9);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.82);
  color: #5182de;
  font-size: 0.68rem;
  font-weight: 800;
}

.error-inline-expand {
  display: inline-grid;
  width: 1.45rem;
  height: 1.45rem;
  place-items: center;
  border: 0;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.82);
  color: #4f82df;
}

.error-tabs-panel {
  margin-top: 1rem;
  border-radius: 2rem 2rem 0 0;
  background:
    radial-gradient(circle at top left, rgba(255, 255, 255, 0.96), transparent 34%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.98) 0%, rgba(247, 250, 255, 0.98) 100%);
  box-shadow:
    0 18px 42px rgba(97, 126, 190, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.96);
}

.error-tabs {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.error-tab {
  position: relative;
  min-height: 3.05rem;
  border: 0;
  background: linear-gradient(180deg, rgba(249, 251, 255, 0.96) 0%, rgba(243, 247, 255, 0.96) 100%);
  color: #8f98ab;
  font-size: 0.88rem;
  font-weight: 900;
}

.error-tab:first-child {
  border-top-left-radius: 2rem;
}

.error-tab:last-child {
  border-top-right-radius: 2rem;
}

.error-tab-active {
  background: white;
  color: #2f7cff;
}

.error-tab-active::after {
  content: "";
  position: absolute;
  left: 50%;
  bottom: 0.28rem;
  width: 2.7rem;
  height: 0.2rem;
  border-radius: 999px;
  background: #2f7cff;
  transform: translateX(-50%);
}

.error-tab-content {
  padding: 0 1.15rem 1rem;
  margin-top: -0.18rem;
}

.error-content-card {
  padding: 1rem 0 0;
}

.error-content-section + .error-content-section {
  margin-top: 0.95rem;
}

.error-content-section-divider {
  padding-top: 0.95rem;
  border-top: 1px solid rgba(231, 237, 248, 0.96);
}

.error-card-head {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  margin-bottom: 0.55rem;
}

.error-card-head h2 {
  margin: 0;
  font-size: 1rem;
  font-weight: 900;
  color: #1a2b57;
}

.error-content-text {
  margin: 0;
  white-space: pre-wrap;
  color: #1a2b57;
  font-size: 0.94rem;
  line-height: 1.72;
}

.error-done-card {
  text-align: center;
}

.error-done-title {
  margin: 0;
  color: #1d2e62;
  font-size: 1.3rem;
  font-weight: 900;
}

.error-done-text {
  margin: 0.6rem auto 0;
  max-width: 20rem;
  color: #6d7ea9;
  font-size: 0.92rem;
  line-height: 1.7;
}

.error-primary-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 3.2rem;
  border: 0;
  border-radius: 1.2rem;
  background: linear-gradient(135deg, #3883ff 0%, #166fff 100%);
  color: white;
  font-size: 0.92rem;
  font-weight: 900;
}

.error-actions {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  display: grid;
  gap: 0.8rem;
  padding: 0.85rem 1.5rem calc(var(--safe-area-bottom) + 0.95rem);
  background: linear-gradient(180deg, rgba(244, 248, 255, 0) 0%, rgba(244, 248, 255, 0.94) 24%, rgba(244, 248, 255, 0.98) 100%);
}

.error-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.45rem;
  min-height: 3.55rem;
  border-radius: 1.2rem;
  border: 0;
  font-size: 0.92rem;
  font-weight: 900;
}

.error-action-wide {
  grid-column: span 2;
}

.error-action-ghost {
  border: 1px solid rgba(205, 219, 244, 0.95);
  background: rgba(255, 255, 255, 0.98);
  color: #263656;
}

.error-action-primary {
  background: linear-gradient(135deg, #3883ff 0%, #166fff 100%);
  color: white;
}

.error-score {
  display: flex;
  min-height: 4rem;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  border: 0;
  border-radius: 1.25rem;
  padding: 0.8rem 0.95rem;
  text-align: left;
  color: white;
}

.error-score-red {
  background: linear-gradient(135deg, #ff8f82 0%, #ff6f66 100%);
}

.error-score-warm {
  background: linear-gradient(135deg, #ffc951 0%, #ffb338 42%, #ffa01f 100%);
}

.error-score-green {
  background: linear-gradient(135deg, #6edd71 0%, #4ed45d 42%, #31c44d 100%);
}

.error-score-title {
  font-size: 0.92rem;
  font-weight: 900;
}

.error-score-hint {
  margin-top: 0.2rem;
  font-size: 0.72rem;
  color: rgba(255, 255, 255, 0.84);
}

@media (min-width: 721px) {
  .error-actions {
    grid-template-columns: 1fr 1.4fr;
  }

  .error-score {
    min-height: 4.3rem;
  }
}

@media (max-width: 720px) {
  .error-header,
  .error-main,
  .error-actions {
    padding-right: 1rem;
    padding-left: 1rem;
  }

  .error-main {
    padding-top: 1rem;
  }

  .error-hero-top {
    grid-template-columns: 5.8rem minmax(0, 1fr);
    gap: 0.95rem;
  }

  .error-thumb {
    height: 8.2rem;
  }

  .error-question-text {
    font-size: 0.94rem;
    line-height: 1.68;
  }

  .error-tab {
    min-height: 2.9rem;
    font-size: 0.86rem;
  }

  .error-tab-content {
    padding-right: 1rem;
    padding-bottom: 0.9rem;
    padding-left: 1rem;
  }

  .error-actions {
    grid-template-columns: 1fr 1.25fr;
  }

  .error-action-wide {
    grid-column: auto;
  }

  .error-actions-rating {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .error-score {
    min-height: 3.8rem;
    padding: 0.7rem 0.8rem;
  }
}
</style>
