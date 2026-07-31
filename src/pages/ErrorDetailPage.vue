<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'
import {
  ArrowLeft,
  Brain,
  ChevronDown,
  ChevronUp,
  Ellipsis,
  Lightbulb,
  NotebookPen,
  PencilLine,
  RefreshCw,
  Save,
  Sparkles,
} from 'lucide-vue-next'
import * as errorApi from '../api/errorItem'
import RichText from '../components/RichText.vue'

type EditableSection = 'question' | 'answer' | 'analysis' | 'mistake' | 'notes'

const route = useRoute()
const router = useRouter()
const item = ref<errorApi.ErrorItem | null>(null)
const conflicts = ref<errorApi.ErrorSyncConflict[]>([])
const saving = ref(false)
const retryingAnalysis = ref(false)
const editingSection = ref<EditableSection | null>(null)
const saveMessage = ref('')
const activeTab = ref<'answer' | 'mistake'>('answer')
const knowledgeExpanded = ref(false)
const knowledgeOverflowing = ref(false)
const localImageFailed = ref(false)
const knowledgeTagsRef = ref<HTMLElement | null>(null)
const questionEditorRef = ref<HTMLTextAreaElement | null>(null)
const answerEditorRef = ref<HTMLTextAreaElement | null>(null)
const analysisEditorRef = ref<HTMLTextAreaElement | null>(null)
const mistakeEditorRef = ref<HTMLTextAreaElement | null>(null)
const notesEditorRef = ref<HTMLTextAreaElement | null>(null)
const form = ref({
  questionText: '',
  answerText: '',
  analysis: '',
  mistakeAnalysis: '',
  userNotes: '',
  knowledgePointsText: '',
})

const imageSrc = computed(() => {
  if (item.value?.localImagePath && !localImageFailed.value) {
    return convertFileSrc(item.value.localImagePath)
  }
  return item.value?.remoteImageUrl || ''
})
const knowledgePoints = computed(() => form.value.knowledgePointsText.split(/[、,\n]/).map(x => x.trim()).filter(Boolean))

let knowledgeResizeObserver: ResizeObserver | null = null

onMounted(async () => {
  await load()
  await nextTick()
  syncAllEditorHeights()
  updateKnowledgeOverflow()
  if (knowledgeTagsRef.value && typeof ResizeObserver !== 'undefined') {
    knowledgeResizeObserver = new ResizeObserver(() => {
      updateKnowledgeOverflow()
    })
    knowledgeResizeObserver.observe(knowledgeTagsRef.value)
  }
})

onBeforeUnmount(() => {
  knowledgeResizeObserver?.disconnect()
  knowledgeResizeObserver = null
})

watch(
  () => [
    knowledgePoints.value.join('|'),
    editingSection.value,
    form.value.questionText,
    form.value.answerText,
    form.value.analysis,
    form.value.mistakeAnalysis,
    form.value.userNotes,
  ],
  async () => {
    await nextTick()
    syncAllEditorHeights()
    updateKnowledgeOverflow()
  },
)

async function load() {
  const [items, nextConflicts] = await Promise.all([
    errorApi.getErrorItems(),
    errorApi.getErrorSyncConflicts(),
  ])
  item.value = items.find(x => x.id === route.params.id) || null
  conflicts.value = nextConflicts
  localImageFailed.value = false
  if (item.value) {
    form.value.questionText = item.value.questionText || ''
    form.value.answerText = item.value.answerText || ''
    form.value.analysis = item.value.analysis || ''
    form.value.mistakeAnalysis = item.value.mistakeAnalysis || ''
    form.value.userNotes = item.value.userNotes || ''
    form.value.knowledgePointsText = errorApi.parseKnowledgePoints(item.value.knowledgePoints).join('、')
    knowledgeExpanded.value = false
  }
}

const itemConflict = computed(() => conflicts.value.find(conflict => conflict.localItemId === item.value?.id) || null)
const canRetryAnalyze = computed(() => item.value?.syncStatus === 'analyze_failed' && Boolean(item.value?.localImagePath))
const conflictBanner = computed(() => {
  if (!itemConflict.value) return null
  switch (itemConflict.value.reason) {
    case 'validation_error':
      return {
        title: '这道题还不能同步',
        text: '服务端拒绝了这次同步。通常是因为 AI 分析还没完成，或图片信息还不完整。',
        showResolveActions: false,
      }
    case 'not_found':
      return {
        title: '远端记录不存在',
        text: '这道题对应的远端记录没有找到。先确认本地内容，再重新同步本地版本。',
        showResolveActions: false,
      }
    case 'version_conflict':
      return {
        title: '这道题和远端版本发生了冲突',
        text: '你本地的修改还在。选一个方向继续同步就行。',
        showResolveActions: true,
      }
    default:
      return {
        title: '这道题暂时无法同步',
        text: '同步时遇到了未分类的问题。先保留本地内容，稍后再试一次更稳妥。',
        showResolveActions: false,
      }
  }
})

function handleImageError() {
  if (item.value?.localImagePath && !localImageFailed.value) {
    localImageFailed.value = true
  }
}

function isEditing(section: EditableSection) {
  return editingSection.value === section
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

  if (!knowledgeOverflowing.value) {
    knowledgeExpanded.value = false
  }
}

function syncEditorHeight(el: HTMLTextAreaElement | null) {
  if (!el) return
  el.style.height = 'auto'
  el.style.height = `${el.scrollHeight}px`
}

function syncAllEditorHeights() {
  syncEditorHeight(questionEditorRef.value)
  syncEditorHeight(answerEditorRef.value)
  syncEditorHeight(analysisEditorRef.value)
  syncEditorHeight(mistakeEditorRef.value)
  syncEditorHeight(notesEditorRef.value)
}

async function save() {
  if (!item.value || saving.value) return
  saving.value = true
  saveMessage.value = ''

  try {
    await errorApi.saveErrorItem({
      id: item.value.id,
      questionText: form.value.questionText,
      answerText: form.value.answerText,
      analysis: form.value.analysis,
      mistakeAnalysis: form.value.mistakeAnalysis,
      userNotes: form.value.userNotes,
      knowledgePoints: knowledgePoints.value,
    })
    await load()
    saveMessage.value = '已保存到本地错题卡'
    editingSection.value = null
  } finally {
    saving.value = false
  }
}

async function toggleSectionEdit(section: EditableSection) {
  if (editingSection.value === section) {
    await save()
    return
  }

  editingSection.value = section
  saveMessage.value = ''

  if (section === 'mistake' || section === 'notes') {
    activeTab.value = 'mistake'
  }
  if (section === 'answer' || section === 'analysis') {
    activeTab.value = 'answer'
  }
}

async function keepLocalVersion() {
  if (!item.value || saving.value) return
  saving.value = true
  try {
    await errorApi.resolveErrorSyncConflictKeepLocal(item.value.id)
    await load()
    saveMessage.value = '已保留本地修改，等待重新同步'
  } finally {
    saving.value = false
  }
}

async function acceptRemoteVersion() {
  if (!item.value || saving.value) return
  saving.value = true
  try {
    await errorApi.resolveErrorSyncConflictAcceptRemote(item.value.id)
    await load()
    saveMessage.value = '已接受远端版本'
  } finally {
    saving.value = false
  }
}

async function retryAnalyze() {
  if (!item.value || retryingAnalysis.value) return
  retryingAnalysis.value = true
  saveMessage.value = ''
  try {
    await errorApi.analyzeErrorDraft(item.value.id)
    await load()
    saveMessage.value = 'AI 分析已重新完成，可继续编辑或同步'
  } catch (e) {
    await load()
    saveMessage.value = e instanceof Error ? e.message : String(e)
  } finally {
    retryingAnalysis.value = false
  }
}
</script>

<template>
  <div class="app-page error-detail-page min-h-screen">
    <header class="error-header header-safe-top">
      <button class="error-header-button" type="button" @click="router.push({ name: 'ErrorNotebook' })">
        <ArrowLeft class="h-6 w-6" />
      </button>
      <h1 class="error-header-title">错题详情</h1>
      <button class="error-header-button" type="button" aria-label="more">
        <Ellipsis class="h-6 w-6" />
      </button>
    </header>

    <main v-if="item" class="error-detail-main">
      <section v-if="itemConflict && conflictBanner" class="mx-4 mt-3 rounded-2xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
        <p class="font-semibold">{{ conflictBanner.title }}</p>
        <p class="mt-1 text-xs leading-5 text-amber-700">{{ conflictBanner.text }}</p>
        <div v-if="conflictBanner.showResolveActions" class="mt-3 flex gap-2">
          <button class="error-secondary-button !h-10 !px-4" type="button" :disabled="saving" @click="keepLocalVersion">
            保留我的修改
          </button>
          <button class="error-primary-button !h-10 !px-4" type="button" :disabled="saving" @click="acceptRemoteVersion">
            接受远端版本
          </button>
        </div>
      </section>

      <section
        v-if="canRetryAnalyze"
        class="mx-4 mt-3 rounded-2xl border border-sky-200 bg-sky-50 px-4 py-3 text-sm text-sky-900"
      >
        <p class="font-semibold">AI 分析失败</p>
        <p class="mt-1 text-xs leading-5 text-sky-800">题目图片和你的编辑内容都还在本地。重新分析成功后，这道题才会进入同步队列。</p>
        <div class="mt-3 flex gap-2">
          <button
            class="error-primary-button !h-10 !px-4"
            type="button"
            :disabled="retryingAnalysis"
            @click="retryAnalyze"
          >
            <RefreshCw class="h-4 w-4" :class="{ 'animate-spin': retryingAnalysis }" />
            {{ retryingAnalysis ? '重新分析中' : '重试 AI 分析' }}
          </button>
        </div>
      </section>

      <section class="error-hero-card">
        <div class="error-hero-top">
          <img
            v-if="imageSrc"
            :src="imageSrc"
            class="error-thumb"
            @error="handleImageError"
          />

          <div class="error-hero-content">
            <div class="error-question-block">
              <div class="error-section-toolbar">
                <button
                  class="error-section-action"
                  type="button"
                  :disabled="saving"
                  :aria-label="isEditing('question') ? '保存题目内容' : '编辑题目内容'"
                  :title="isEditing('question') ? '保存题目内容' : '编辑题目内容'"
                  @click="toggleSectionEdit('question')"
                >
                  <Save v-if="isEditing('question')" class="h-4 w-4" />
                  <PencilLine v-else class="h-4 w-4" />
                </button>
              </div>

              <RichText
                v-if="!isEditing('question')"
                class="error-question-editor"
                :text="form.questionText"
                fallback="把题干整理成自己一眼能读懂的版本"
              />
              <textarea
                v-else
                ref="questionEditorRef"
                v-model="form.questionText"
                class="error-editor error-question-editor"
                placeholder="把题干整理成自己一眼能读懂的版本"
                @input="syncEditorHeight(questionEditorRef)"
              />

              <input
                v-if="isEditing('question')"
                v-model="form.knowledgePointsText"
                class="error-knowledge-input"
                placeholder="用 、 或逗号分隔知识点"
              />

              <div
                v-if="knowledgePoints.length"
                class="error-question-tags-row"
                :class="{ 'error-question-tags-row-editing': isEditing('question') }"
              >
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
        </div>
      </section>

      <section class="error-tabs-panel">
        <div class="error-tabs">
          <button
            class="error-tab"
            :class="{ 'error-tab-active': activeTab === 'answer' }"
            type="button"
            @click="activeTab = 'answer'"
          >
            答案与解析
          </button>
          <button
            class="error-tab"
            :class="{ 'error-tab-active': activeTab === 'mistake' }"
            type="button"
            @click="activeTab = 'mistake'"
          >
            错因与笔记
          </button>
        </div>

        <div class="error-tab-content">
          <template v-if="activeTab === 'answer'">
            <article class="error-content-card">
              <section class="error-content-section">
                <div class="error-card-head error-card-head-between">
                  <div class="error-card-head">
                    <Sparkles class="h-5 w-5 text-[#2f7cff]" />
                    <h2>标准答案</h2>
                  </div>
                  <button
                    class="error-section-action"
                    type="button"
                    :disabled="saving"
                    :aria-label="isEditing('answer') ? '保存标准答案' : '编辑标准答案'"
                    :title="isEditing('answer') ? '保存标准答案' : '编辑标准答案'"
                    @click="toggleSectionEdit('answer')"
                  >
                    <Save v-if="isEditing('answer')" class="h-4 w-4" />
                    <PencilLine v-else class="h-4 w-4" />
                  </button>
                </div>
                <RichText
                  v-if="!isEditing('answer')"
                  class="error-content-text"
                  :text="form.answerText"
                  fallback="暂无答案"
                />
                <textarea
                  v-else
                  ref="answerEditorRef"
                  v-model="form.answerText"
                  class="error-editor error-content-editor error-answer-editor"
                  placeholder="用最短路径写出标准答案"
                  @input="syncEditorHeight(answerEditorRef)"
                />
              </section>

              <section class="error-content-section error-content-section-divider">
                <div class="error-card-head error-card-head-between">
                  <div class="error-card-head">
                    <Lightbulb class="h-5 w-5 text-[#2f7cff]" />
                    <h2>解析</h2>
                  </div>
                  <button
                    class="error-section-action"
                    type="button"
                    :disabled="saving"
                    :aria-label="isEditing('analysis') ? '保存解析' : '编辑解析'"
                    :title="isEditing('analysis') ? '保存解析' : '编辑解析'"
                    @click="toggleSectionEdit('analysis')"
                  >
                    <Save v-if="isEditing('analysis')" class="h-4 w-4" />
                    <PencilLine v-else class="h-4 w-4" />
                  </button>
                </div>
                <RichText
                  v-if="!isEditing('analysis')"
                  class="error-content-text"
                  :text="form.analysis"
                  fallback="暂无解析"
                />
                <textarea
                  v-else
                  ref="analysisEditorRef"
                  v-model="form.analysis"
                  class="error-editor error-content-editor error-analysis-editor"
                  placeholder="把关键思路、公式、判断步骤串成一段顺畅的解析"
                  @input="syncEditorHeight(analysisEditorRef)"
                />
              </section>
            </article>
          </template>

          <template v-else>
            <article class="error-content-card">
              <section v-if="item.wrongAnswerText" class="error-content-section">
                <div class="error-card-head">
                  <Brain class="h-5 w-5 text-[#ff786d]" />
                  <h2>错答记录</h2>
                </div>
                <RichText class="error-content-text" :text="item.wrongAnswerText" />
              </section>

              <section class="error-content-section" :class="{ 'error-content-section-divider': item.wrongAnswerText }">
                <div class="error-card-head error-card-head-between">
                  <div class="error-card-head">
                    <Brain class="h-5 w-5 text-[#ff786d]" />
                    <h2>错因分析</h2>
                  </div>
                  <button
                    class="error-section-action"
                    type="button"
                    :disabled="saving"
                    :aria-label="isEditing('mistake') ? '保存错因分析' : '编辑错因分析'"
                    :title="isEditing('mistake') ? '保存错因分析' : '编辑错因分析'"
                    @click="toggleSectionEdit('mistake')"
                  >
                    <Save v-if="isEditing('mistake')" class="h-4 w-4" />
                    <PencilLine v-else class="h-4 w-4" />
                  </button>
                </div>
                <RichText
                  v-if="!isEditing('mistake')"
                  class="error-content-text"
                  :text="form.mistakeAnalysis"
                  fallback="暂无错因分析"
                />
                <textarea
                  v-else
                  ref="mistakeEditorRef"
                  v-model="form.mistakeAnalysis"
                  class="error-editor error-content-editor error-note-editor"
                  placeholder="记录这次真正卡住的点：概念混淆、漏条件、粗心还是步骤断裂"
                  @input="syncEditorHeight(mistakeEditorRef)"
                />
              </section>

              <section class="error-content-section error-content-section-divider">
                <div class="error-card-head error-card-head-between">
                  <div class="error-card-head">
                    <NotebookPen class="h-5 w-5 text-[#2f7cff]" />
                    <h2>笔记</h2>
                  </div>
                  <button
                    class="error-section-action"
                    type="button"
                    :disabled="saving"
                    :aria-label="isEditing('notes') ? '保存笔记' : '编辑笔记'"
                    :title="isEditing('notes') ? '保存笔记' : '编辑笔记'"
                    @click="toggleSectionEdit('notes')"
                  >
                    <Save v-if="isEditing('notes')" class="h-4 w-4" />
                    <PencilLine v-else class="h-4 w-4" />
                  </button>
                </div>
                <RichText
                  v-if="!isEditing('notes')"
                  class="error-content-text"
                  :text="form.userNotes"
                  fallback="暂无笔记"
                />
                <textarea
                  v-else
                  ref="notesEditorRef"
                  v-model="form.userNotes"
                  class="error-editor error-content-editor error-note-editor"
                  placeholder="写一句下次看到这类题先做什么、先检查什么"
                  @input="syncEditorHeight(notesEditorRef)"
                />
              </section>
            </article>
          </template>
        </div>
      </section>

      <p v-if="saveMessage" class="error-save-message">{{ saveMessage }}</p>
    </main>

    <main v-else class="error-empty-wrap">
      <section class="error-empty-card">
        <p class="error-empty-title">这道错题还没找到</p>
        <p class="error-empty-text">可能已经被删除，或者还没同步到本地。</p>
      </section>
    </main>
  </div>
</template>

<style scoped>
.error-detail-page {
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
  font-size: 0.92rem;
  font-weight: 900;
  color: #18274f;
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

.error-detail-main {
  padding: 1.15rem 1.5rem 1.75rem;
}

.error-hero-card {
  position: relative;
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

.error-hero-top {
  display: grid;
  grid-template-columns: 10.2rem minmax(0, 1fr);
  gap: 1.25rem;
  padding: 1.35rem 1.35rem 1.2rem;
}

.error-thumb {
  width: 100%;
  height: 12.4rem;
  object-fit: cover;
  border-radius: 1.15rem;
  border: 3px solid rgba(255, 255, 255, 0.96);
  box-shadow: 0 14px 28px rgba(106, 127, 176, 0.12);
}

.error-hero-content {
  min-width: 0;
}

.error-question-block {
  display: flex;
  min-height: 100%;
  flex-direction: column;
  justify-content: center;
}

.error-section-toolbar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  margin-bottom: 0.05rem;
}

.error-section-action {
  display: inline-grid;
  width: 1.95rem;
  height: 1.95rem;
  place-items: center;
  border: 1px solid rgba(198, 216, 255, 0.92);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.82);
  color: #2f7cff;
}

.error-knowledge-input {
  width: 100%;
  height: 2.35rem;
  margin-top: 0.18rem;
  border: 1px solid rgba(199, 216, 249, 0.9);
  border-radius: 0.95rem;
  background: rgba(255, 255, 255, 0.86);
  padding: 0 0.9rem;
  color: #1c2b61;
  outline: none;
}

.error-question-tags-row {
  display: flex;
  align-items: flex-start;
  gap: 0.35rem;
  margin-top: 0;
}

.error-question-tags-row-editing {
  margin-top: 0.16rem;
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
  overflow: hidden;
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
  gap: 0;
}

.error-tab {
  position: relative;
  min-height: 3.15rem;
  border: 0;
  background: linear-gradient(180deg, rgba(249, 251, 255, 0.96) 0%, rgba(243, 247, 255, 0.96) 100%);
  color: #8f98ab;
  font-size: 0.9rem;
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
  width: 2.8rem;
  height: 0.2rem;
  border-radius: 999px;
  background: #2f7cff;
  transform: translateX(-50%);
}

.error-tab-content {
  display: grid;
  gap: 0.85rem;
  padding: 0 1.15rem 1rem;
  margin-top: -0.18rem;
}

.error-content-card {
  padding: 1.05rem 0 0;
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

.error-card-head-between {
  justify-content: space-between;
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
  font-size: 0.94rem;
  line-height: 1.72;
  color: #1a2b57;
}

.error-editor {
  width: 100%;
  resize: none;
  border: 0;
  background: transparent;
  padding: 0;
  outline: none;
  color: #1a2b57;
  overflow: hidden;
}

.error-editor-readonly {
  pointer-events: none;
}

.error-editor::placeholder {
  color: #9fb0d1;
}

.error-question-editor {
  min-height: 0;
  font-size: 1rem;
  line-height: 1.78;
}

.error-content-editor {
  min-height: 0;
  font-size: 0.94rem;
  line-height: 1.72;
}

.error-save-message {
  margin: 0.95rem 0 0;
  border-radius: 1rem;
  background: rgba(236, 251, 242, 0.98);
  padding: 0.85rem 1rem;
  color: #23a35d;
  font-size: 0.92rem;
  font-weight: 800;
}

.error-empty-wrap {
  padding: 1.2rem 1.5rem 8rem;
}

.error-empty-card {
  border-radius: 1.8rem;
  background: rgba(255, 255, 255, 0.94);
  padding: 2rem 1.5rem;
  text-align: center;
  box-shadow: 0 16px 32px rgba(95, 126, 194, 0.06);
}

.error-empty-title {
  margin: 0;
  color: #1c2b61;
  font-size: 1.05rem;
  font-weight: 900;
}

.error-empty-text {
  margin: 0.55rem 0 0;
  color: #92a1c7;
  font-size: 0.92rem;
}

@media (max-width: 720px) {
  .error-hero-top {
    grid-template-columns: 5.8rem minmax(0, 1fr);
    gap: 0.95rem;
    padding: 1rem 1rem 0.95rem;
  }

  .error-thumb {
    height: 8.2rem;
  }

  .error-question-editor {
    font-size: 0.94rem;
    line-height: 1.68;
  }

  .error-question-tags-row {
    margin-top: 0;
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
}
</style>
