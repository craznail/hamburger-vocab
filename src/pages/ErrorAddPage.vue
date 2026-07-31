<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { convertFileSrc } from '@tauri-apps/api/core'
import {
  ArrowLeft,
  Brain,
  Check,
  ChevronDown,
  Ellipsis,
  ImagePlus,
  Lightbulb,
  Loader,
  Lock,
  NotebookPen,
  Sparkles,
  WandSparkles,
} from 'lucide-vue-next'
import * as errorApi from '../api/errorItem'
import { useErrorNotebookStore } from '../stores/useErrorNotebookStore'

const router = useRouter()
const notebookStore = useErrorNotebookStore()
const { notebooks, loading } = storeToRefs(notebookStore)
const draft = ref<errorApi.ErrorDraft | null>(null)
const item = ref<errorApi.ErrorItem | null>(null)
const status = ref('')
const busy = ref(false)
const notebookMessage = ref('')
const selectedNotebookId = ref('')
const activeTab = ref<'answer' | 'mistake'>('answer')
const form = ref({
  questionText: '',
  answerText: '',
  analysis: '',
  mistakeAnalysis: '',
  userNotes: '',
  knowledgePointsText: '',
})

const imageSrc = computed(() => draft.value?.localImagePath ? convertFileSrc(draft.value.localImagePath) : '')
const knowledgePoints = computed(() => form.value.knowledgePointsText.split(/[、,\n]/).map(x => x.trim()).filter(Boolean))
const selectedNotebook = computed(() => notebooks.value.find(notebook => notebook.id === selectedNotebookId.value) || null)
const notebookLocked = computed(() => Boolean(draft.value))
const hasNotebookLoadError = computed(() => Boolean(notebookMessage.value) && notebooks.value.length === 0)
const canChooseImage = computed(() => !busy.value && !hasNotebookLoadError.value && Boolean(selectedNotebookId.value) && notebooks.value.length > 0)
const selectorHint = computed(() => {
  if (hasNotebookLoadError.value) return notebookMessage.value
  if (!selectedNotebook.value) return loading.value ? '正在加载错题本...' : '还没有可用错题本，暂时无法上传题目图片'
  return notebookLocked.value
    ? '本次新增的图片和分析结果都会保存在这里'
    : '上传前可以切换，创建草稿后会锁定归属'
})
const selectorStatusText = computed(() => notebookLocked.value ? '已锁定' : '可切换')
const stageTitle = computed(() => {
  if (busy && !item.value) return '正在让 AI 整理这道题'
  if (item.value) return 'AI 已整理完成，确认后保存'
  if (draft.value) return '草稿已生成，等待分析结果'
  return '先上传图片，生成一张可编辑的错题卡'
})
const stageHint = computed(() => {
  if (busy && !item.value) return status.value || '图片已保存到本地，正在调用服务端 AI'
  if (item.value) return '你可以直接顺着这张卡片确认题干、答案、解析和错因'
  if (draft.value) return status.value || '草稿已创建'
  return '建议先拍完整题面，后续可以在卡片里再细修内容'
})

watch(notebooks, (nextNotebooks) => {
  if (notebookLocked.value) return
  if (selectedNotebookId.value && nextNotebooks.some(notebook => notebook.id === selectedNotebookId.value)) {
    return
  }
  selectedNotebookId.value = nextNotebooks[0]?.id || ''
}, { immediate: true })

onMounted(async () => {
  try {
    await notebookStore.ensureFresh()
    notebookMessage.value = ''
  } catch (e) {
    notebookMessage.value = e instanceof Error ? e.message : String(e)
  }
})

function readAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(String(reader.result))
    reader.onerror = () => reject(reader.error)
    reader.readAsDataURL(file)
  })
}

async function chooseImage(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  if (!selectedNotebookId.value) {
    status.value = notebookMessage.value || '请先选择错题本'
    input.value = ''
    return
  }

  busy.value = true
  status.value = '正在保存本地草稿...'

  try {
    const dataUrl = await readAsDataUrl(file)
    draft.value = await errorApi.createErrorDraft(dataUrl, file.type || 'image/jpeg', selectedNotebookId.value)
    status.value = '本地草稿已保存，开始 AI 分析...'
    item.value = await errorApi.analyzeErrorDraft(draft.value.id)
    form.value.questionText = item.value.questionText || ''
    form.value.answerText = item.value.answerText || ''
    form.value.analysis = item.value.analysis || ''
    form.value.mistakeAnalysis = item.value.mistakeAnalysis || ''
    form.value.userNotes = item.value.userNotes || ''
    form.value.knowledgePointsText = errorApi.parseKnowledgePoints(item.value.knowledgePoints).join('、')
    status.value = 'AI 分析完成，可编辑后保存'
  } catch (e) {
    status.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = false
    input.value = ''
  }
}

async function save() {
  if (!draft.value || busy.value) return
  busy.value = true

  try {
    await errorApi.saveErrorItem({
      id: draft.value.id,
      questionText: form.value.questionText,
      answerText: form.value.answerText,
      analysis: form.value.analysis,
      mistakeAnalysis: form.value.mistakeAnalysis,
      userNotes: form.value.userNotes,
      knowledgePoints: knowledgePoints.value,
    })
    router.push({ name: 'ErrorDetail', params: { id: draft.value.id } })
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div class="app-page add-page min-h-screen">
    <header class="error-header header-safe-top">
      <button class="error-header-button" type="button" @click="router.push({ name: 'ErrorNotebook' })">
        <ArrowLeft class="h-6 w-6" />
      </button>
      <div class="text-center">
        <h1 class="error-header-title">添加错题</h1>
        <p class="error-header-subtitle">上传图片后，整理成一张能复习的错题卡</p>
      </div>
      <button class="error-header-button" type="button" aria-label="more">
        <Ellipsis class="h-6 w-6" />
      </button>
    </header>

    <main class="error-main">
      <section class="error-hero-card">
        <div class="error-stage-head">
          <div class="error-stage-icon">
            <WandSparkles class="h-5 w-5" />
          </div>
          <div>
            <p class="error-kicker">AI 错题卡</p>
            <h2 class="error-stage-title">{{ stageTitle }}</h2>
            <p class="error-stage-text">{{ stageHint }}</p>
          </div>
        </div>

        <label class="error-select-block">
          <div class="error-select-head">
            <span class="error-select-label">保存到错题本</span>
            <span class="error-select-status" :class="{ 'error-select-status-locked': notebookLocked }">
              <Lock v-if="notebookLocked" class="h-3.5 w-3.5" />
              <NotebookPen v-else class="h-3.5 w-3.5" />
              {{ selectorStatusText }}
            </span>
          </div>

          <div
            class="error-select-shell"
            :class="{
              'error-select-shell-disabled': busy || notebookLocked,
              'error-select-shell-warning': hasNotebookLoadError,
            }"
          >
            <div class="error-select-icon">
              <NotebookPen class="h-4 w-4" />
            </div>

            <div class="error-select-copy">
              <p class="error-select-value">{{ selectedNotebook?.name || (loading ? '正在加载错题本...' : '暂无可用错题本') }}</p>
              <p class="error-select-caption">
                {{ selectedNotebook ? 'AI 分析与后续保存会直接归入这个错题本' : '需要先拿到一个可用错题本才能继续上传' }}
              </p>
            </div>

            <ChevronDown class="error-select-chevron h-4 w-4" />

            <select
              v-model="selectedNotebookId"
              class="error-select"
              :disabled="busy || notebookLocked"
            >
              <option v-if="!notebooks.length" value="" disabled>
                {{ loading ? '正在加载错题本...' : '暂无可用错题本' }}
              </option>
              <option v-for="notebook in notebooks" :key="notebook.id" :value="notebook.id">
                {{ notebook.name }}
              </option>
            </select>
          </div>

          <p class="error-select-message" :class="{ 'error-select-message-warning': hasNotebookLoadError }">
            {{ selectorHint }}
          </p>
        </label>

        <div class="error-stage-actions">
          <label class="error-primary-button" :class="{ 'error-primary-button-disabled': !canChooseImage }">
            <ImagePlus class="h-4 w-4" />
            {{ draft ? '重新选择图片' : '选择题目图片' }}
            <input class="hidden" type="file" accept="image/*" :disabled="!canChooseImage" @change="chooseImage" />
          </label>
          <div class="error-chip-button">
            <Loader v-if="busy" class="h-4 w-4 animate-spin" />
            <Sparkles v-else class="h-4 w-4" />
            {{ busy ? '分析中' : '本地优先保存' }}
          </div>
        </div>

        <p v-if="status" class="error-message">{{ status }}</p>

        <img v-if="imageSrc" :src="imageSrc" class="error-stage-image" />
      </section>

      <template v-if="draft">
        <section class="error-hero-card error-question-card">
          <div class="error-question-layout">
            <img v-if="imageSrc" :src="imageSrc" class="error-thumb" />
            <div class="error-question-block">
              <textarea
                v-model="form.questionText"
                class="error-editor error-question-editor"
                placeholder="把题目改成自己一眼能读懂的版本"
              />
              <input
                v-model="form.knowledgePointsText"
                class="error-input"
                placeholder="用 、 或逗号分隔知识点"
              />
              <div v-if="knowledgePoints.length" class="error-chip-row">
                <span v-for="point in knowledgePoints" :key="point" class="error-chip">{{ point }}</span>
              </div>
            </div>
          </div>
        </section>

        <section class="error-tabs-panel">
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
                  <textarea
                    v-model="form.answerText"
                    class="error-editor error-content-editor error-answer-editor"
                    placeholder="补成自己复习时最需要的答案表达"
                  />
                </section>

                <section class="error-content-section error-content-section-divider">
                  <div class="error-card-head">
                    <Lightbulb class="h-5 w-5 text-[#2f7cff]" />
                    <h2>解析</h2>
                  </div>
                  <textarea
                    v-model="form.analysis"
                    class="error-editor error-content-editor error-analysis-editor"
                    placeholder="把解题思路、公式依据、容易漏掉的判断放在这里"
                  />
                </section>
              </template>

              <template v-else>
                <section v-if="item?.wrongAnswerText" class="error-content-section">
                  <div class="error-card-head">
                    <Brain class="h-5 w-5 text-[#ff786d]" />
                    <h2>识别到的错误答案</h2>
                  </div>
                  <p class="error-content-text">{{ item.wrongAnswerText }}</p>
                </section>

                <section class="error-content-section" :class="{ 'error-content-section-divider': item?.wrongAnswerText }">
                  <div class="error-card-head">
                    <Brain class="h-5 w-5 text-[#ff786d]" />
                    <h2>错因分析</h2>
                  </div>
                  <textarea
                    v-model="form.mistakeAnalysis"
                    class="error-editor error-content-editor error-note-editor"
                    placeholder="说明这题为什么会错：概念不清、公式乱用、条件漏看、计算粗心..."
                  />
                </section>

                <section class="error-content-section error-content-section-divider">
                  <div class="error-card-head">
                    <NotebookPen class="h-5 w-5 text-[#2f7cff]" />
                    <h2>我的提醒</h2>
                  </div>
                  <textarea
                    v-model="form.userNotes"
                    class="error-editor error-content-editor error-note-editor"
                    placeholder="给下次复习的自己留一句提醒：先看定义、先列已知、先验算哪一步"
                  />
                </section>
              </template>
            </article>
          </div>
        </section>
      </template>
    </main>

    <footer class="error-actions">
      <label class="error-action error-action-ghost" :class="{ 'error-action-disabled': !canChooseImage }">
        重新选图
        <input class="hidden" type="file" accept="image/*" :disabled="!canChooseImage" @change="chooseImage" />
      </label>
      <button class="error-action error-action-warm" type="button" @click="router.push({ name: 'ErrorNotebook' })">
        稍后再说
      </button>
      <button class="error-action error-action-primary" type="button" :disabled="busy || !draft" @click="save">
        <Check class="h-5 w-5" />
        {{ busy ? '保存中' : '保存错题' }}
      </button>
    </footer>
  </div>
</template>

<style scoped>
.add-page {
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
  background: rgba(255, 255, 255, 0.72);
  color: #23345f;
  box-shadow: 0 10px 24px rgba(94, 122, 190, 0.08);
}

.error-main {
  padding: 1.15rem 1.5rem 8.8rem;
}

.error-hero-card,
.error-tabs-panel {
  border: 1px solid rgba(214, 227, 255, 0.95);
  border-radius: 2rem;
  background:
    radial-gradient(circle at top right, rgba(255, 255, 255, 0.44), transparent 30%),
    linear-gradient(180deg, #eef5ff 0%, #eaf2ff 100%);
  box-shadow:
    0 20px 44px rgba(93, 123, 194, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.96);
}

.error-hero-card {
  padding: 1.35rem;
}

.error-stage-head {
  display: flex;
  align-items: flex-start;
  gap: 0.9rem;
}

.error-stage-icon {
  display: grid;
  width: 3rem;
  height: 3rem;
  place-items: center;
  border-radius: 1rem;
  background: linear-gradient(180deg, #eef4ff 0%, #e5efff 100%);
  color: #3564ff;
  box-shadow: 0 12px 22px rgba(74, 114, 255, 0.12);
}

.error-kicker {
  margin: 0;
  color: #4a7be0;
  font-size: 0.82rem;
  font-weight: 800;
}

.error-stage-title {
  margin: 0.35rem 0 0;
  color: #1d2e62;
  font-size: 1.15rem;
  font-weight: 900;
}

.error-stage-text {
  margin: 0.45rem 0 0;
  color: #6d7ea9;
  font-size: 0.9rem;
  line-height: 1.65;
}

.error-stage-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  margin-top: 1rem;
}

.error-select-block {
  display: block;
  margin-top: 1.1rem;
}

.error-select-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  margin-bottom: 0.55rem;
}

.error-select-label {
  color: #5872ad;
  font-size: 0.8rem;
  font-weight: 800;
}

.error-select-status {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  min-height: 1.7rem;
  padding: 0 0.65rem;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.72);
  color: #4a7be0;
  font-size: 0.72rem;
  font-weight: 800;
  box-shadow: inset 0 0 0 1px rgba(193, 211, 248, 0.88);
}

.error-select-status-locked {
  color: #6f7ea5;
}

.error-select-shell {
  position: relative;
  display: grid;
  grid-template-columns: 2.9rem minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.85rem;
  min-height: 4.55rem;
  border-radius: 1.25rem;
  border: 1px solid rgba(186, 206, 246, 0.96);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.92) 0%, rgba(246, 250, 255, 0.84) 100%);
  padding: 0.8rem 0.95rem;
  box-shadow:
    0 12px 26px rgba(111, 137, 196, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.96);
  overflow: hidden;
}

.error-select-shell-disabled {
  background:
    linear-gradient(180deg, rgba(248, 250, 255, 0.96) 0%, rgba(241, 246, 255, 0.92) 100%);
}

.error-select-shell-warning {
  border-color: rgba(236, 176, 151, 0.9);
}

.error-select-icon {
  display: grid;
  width: 2.9rem;
  height: 2.9rem;
  place-items: center;
  border-radius: 0.95rem;
  background: linear-gradient(180deg, rgba(233, 240, 255, 0.92) 0%, rgba(225, 235, 255, 0.82) 100%);
  color: #3a6fff;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.96);
}

.error-select-copy {
  min-width: 0;
}

.error-select-value {
  margin: 0;
  color: #203468;
  font-size: 1rem;
  font-weight: 900;
  line-height: 1.2;
}

.error-select-caption {
  margin: 0.28rem 0 0;
  color: #7990be;
  font-size: 0.78rem;
  line-height: 1.45;
}

.error-select-chevron {
  color: #5f79b7;
  flex: none;
}

.error-select {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  border: 0;
  border-radius: 1.25rem;
  background: transparent;
  opacity: 0;
  cursor: pointer;
  outline: none;
  appearance: none;
  -webkit-appearance: none;
}

.error-select:disabled {
  cursor: default;
}

.error-select-message {
  margin: 0.55rem 0 0;
  color: #6d7ea9;
  font-size: 0.76rem;
  line-height: 1.5;
}

.error-select-message-warning {
  color: #c06c52;
}

.error-primary-button,
.error-chip-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.45rem;
  min-height: 3rem;
  padding: 0 1rem;
  border-radius: 1.1rem;
  font-size: 0.9rem;
  font-weight: 900;
}

.error-primary-button {
  background: linear-gradient(135deg, #3883ff 0%, #166fff 100%);
  color: white;
  box-shadow: 0 16px 30px rgba(22, 111, 255, 0.26);
}

.error-chip-button {
  border: 1px solid rgba(198, 216, 255, 0.92);
  background: rgba(255, 255, 255, 0.84);
  color: #2f7cff;
}

.error-message {
  margin: 1rem 0 0;
  border-radius: 1rem;
  background: rgba(255, 255, 255, 0.8);
  padding: 0.85rem 1rem;
  color: #6c7ea8;
  font-size: 0.85rem;
  font-weight: 700;
}

.error-stage-image,
.error-thumb {
  width: 100%;
  object-fit: cover;
  border-radius: 1.2rem;
  border: 3px solid rgba(255, 255, 255, 0.96);
  box-shadow: 0 14px 28px rgba(106, 127, 176, 0.12);
}

.error-stage-image {
  max-height: 18rem;
  margin-top: 1rem;
}

.error-question-card {
  margin-top: 1rem;
}

.error-question-layout {
  display: grid;
  grid-template-columns: 7.2rem minmax(0, 1fr);
  gap: 1rem;
}

.error-thumb {
  height: 9.6rem;
}

.error-question-block {
  min-width: 0;
}

.error-input,
.error-editor {
  width: 100%;
  border: 0;
  outline: none;
  resize: none;
  background: rgba(255, 255, 255, 0.82);
  color: #1a2b57;
}

.error-input {
  height: 2.8rem;
  margin-top: 0.9rem;
  border-radius: 1rem;
  border: 1px solid rgba(199, 216, 249, 0.9);
  padding: 0 1rem;
}

.error-editor {
  min-height: 10.5rem;
  border-radius: 1.2rem;
  padding: 0.95rem 1rem;
  font-size: 0.98rem;
  line-height: 1.72;
}

.error-chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.55rem;
  margin-top: 0.25rem;
}

.error-chip {
  display: inline-flex;
  align-items: center;
  min-height: 1.6rem;
  padding: 0 0.65rem;
  border: 1px solid rgba(188, 212, 255, 0.9);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.82);
  color: #5182de;
  font-size: 0.72rem;
  font-weight: 800;
}

.error-tabs-panel {
  margin-top: 1rem;
  overflow: hidden;
}

.error-tabs {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.error-tab {
  min-height: 3.1rem;
  border: 0;
  background: linear-gradient(180deg, rgba(249, 251, 255, 0.96) 0%, rgba(243, 247, 255, 0.96) 100%);
  color: #8f98ab;
  font-size: 0.9rem;
  font-weight: 900;
}

.error-tab-active {
  background: white;
  color: #2f7cff;
}

.error-tab-content {
  padding: 0 1.15rem 1rem;
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
  margin-bottom: 0.7rem;
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

.error-content-editor {
  padding: 0;
  background: transparent;
  font-size: 0.94rem;
  line-height: 1.72;
}

.error-answer-editor {
  min-height: 5.4rem;
}

.error-analysis-editor {
  min-height: 10rem;
}

.error-note-editor {
  min-height: 8rem;
}

.error-actions {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  display: grid;
  grid-template-columns: 1fr 1fr 1.1fr;
  gap: 0.8rem;
  padding: 0.85rem 1.5rem calc(var(--safe-area-bottom) + 0.95rem);
  background: linear-gradient(180deg, rgba(244, 248, 255, 0) 0%, rgba(244, 248, 255, 0.94) 24%, rgba(244, 248, 255, 0.98) 100%);
}

.error-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.45rem;
  min-height: 3.7rem;
  border-radius: 1.25rem;
  border: 0;
  font-size: 0.94rem;
  font-weight: 900;
}

.error-action-ghost {
  border: 1px solid rgba(205, 219, 244, 0.95);
  background: rgba(255, 255, 255, 0.98);
  color: #263656;
}

.error-action-warm {
  background: linear-gradient(135deg, #ff8f82 0%, #ff6f66 100%);
  color: white;
}

.error-action-primary {
  background: linear-gradient(135deg, #3883ff 0%, #166fff 100%);
  color: white;
}

.error-primary-button-disabled,
.error-action-disabled {
  opacity: 0.56;
  pointer-events: none;
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

  .error-question-layout {
    grid-template-columns: 5.6rem minmax(0, 1fr);
  }

  .error-thumb {
    height: 7.6rem;
  }
}
</style>
