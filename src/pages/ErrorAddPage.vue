<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'
import {
  Brain,
  Check,
  ImagePlus,
  Lightbulb,
  Loader,
  NotebookPen,
  Sparkles,
  Tag,
  WandSparkles,
} from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
import * as errorApi from '../api/errorItem'

const router = useRouter()
const draft = ref<errorApi.ErrorDraft | null>(null)
const item = ref<errorApi.ErrorItem | null>(null)
const status = ref('')
const busy = ref(false)
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
const stageTitle = computed(() => {
  if (busy && !item.value) return '正在让 AI 整理这道题'
  if (item.value) return 'AI 已整理完成，确认后保存'
  if (draft.value) return '草稿已生成，等待分析结果'
  return '先上传图片，生成一张可编辑的错题卡'
})
const stageHint = computed(() => {
  if (busy && !item.value) return status.value || '图片已保存到本地，正在调用服务端 AI'
  if (item.value) return '你可以直接顺着这张卡片检查题干、答案、解析和错因'
  if (draft.value) return status.value || '草稿已创建'
  return '建议先拍完整题面，后续可以在卡片里再细修内容'
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

  busy.value = true
  status.value = '正在保存本地草稿...'

  try {
    const dataUrl = await readAsDataUrl(file)
    draft.value = await errorApi.createErrorDraft(dataUrl, file.type || 'image/jpeg')
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
  if (!draft.value) return
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
  <div class="app-page min-h-screen">
    <NavBar @back="router.push({ name: 'ErrorNotebook' })">
      <template #left>
        <div>
          <h1 class="text-sm font-black text-ink">添加错题</h1>
          <p class="mt-1 text-xs text-slate-400">上传图片后，整理成一张能复习的错题卡</p>
        </div>
      </template>
    </NavBar>

    <main class="grid gap-4 px-4 pt-4">
      <section class="add-hero soft-panel overflow-hidden p-4">
        <div class="grid gap-4">
          <div class="flex items-start gap-3">
            <div class="icon-well add-hero-icon">
              <WandSparkles class="h-5 w-5" />
            </div>
            <div class="min-w-0">
              <p class="section-kicker text-blue-500/90">AI 错题卡</p>
              <h2 class="mt-1 text-xl font-black leading-tight text-ink">{{ stageTitle }}</h2>
              <p class="mt-2 text-sm leading-6 text-slate-500">{{ stageHint }}</p>
            </div>
          </div>

          <div class="flex flex-wrap gap-3">
            <label class="blue-gradient inline-flex h-12 items-center justify-center gap-2 rounded-2xl px-5 text-sm font-bold text-white shadow-[0_16px_28px_rgba(53,100,255,0.22)]">
              <ImagePlus class="h-4 w-4" />
              {{ draft ? '重新选择图片' : '选择题目图片' }}
              <input class="hidden" type="file" accept="image/*" @change="chooseImage" />
            </label>
            <div class="ghost-button min-h-12 rounded-2xl px-4 text-xs font-black">
              <Loader v-if="busy" class="h-4 w-4 animate-spin text-blue-500" />
              <Sparkles v-else class="h-4 w-4 text-blue-500" />
              {{ busy ? '分析中' : '本地优先保存' }}
            </div>
          </div>

          <p v-if="status" class="rounded-[22px] bg-white/82 px-4 py-3 text-sm font-semibold text-slate-500 shadow-[inset_0_1px_0_rgba(255,255,255,0.92)]">
            {{ status }}
          </p>

          <img v-if="imageSrc" :src="imageSrc" class="max-h-80 w-full rounded-[24px] object-contain bg-white/92 shadow-[0_18px_38px_rgba(83,116,191,0.12)]" />
        </div>
      </section>

      <section v-if="draft" class="soft-panel overflow-hidden p-0">
        <div class="detail-band detail-band-topic p-4">
          <div class="flex items-center gap-3">
            <div class="icon-well bg-white/22 text-white">
              <Sparkles class="h-4 w-4" />
            </div>
            <div>
              <p class="text-xs font-black uppercase tracking-[0.16em] text-white/74">题目理解</p>
              <h2 class="text-lg font-black text-white">先把题干整理顺</h2>
            </div>
          </div>
        </div>

        <div class="p-4">
          <div class="detail-surface">
            <textarea
              v-model="form.questionText"
              class="detail-textarea min-h-36 text-base font-black leading-7 text-ink"
              placeholder="把题目改成自己一眼能读懂的版本"
            />
          </div>
        </div>
      </section>

      <section v-if="draft" class="soft-panel overflow-hidden p-0">
        <div class="detail-band detail-band-answer p-4">
          <div class="flex items-center gap-3">
            <div class="icon-well bg-white/22 text-white">
              <Lightbulb class="h-4 w-4" />
            </div>
            <div>
              <p class="text-xs font-black uppercase tracking-[0.16em] text-white/74">解题路径</p>
              <h2 class="text-lg font-black text-white">答案和解析一起确认</h2>
            </div>
          </div>
        </div>

        <div class="grid gap-4 p-4">
          <div class="detail-surface">
            <p class="mb-3 text-xs font-black uppercase tracking-[0.14em] text-blue-500">标准答案</p>
            <textarea
              v-model="form.answerText"
              class="detail-textarea min-h-20 text-sm leading-6 text-ink"
              placeholder="补成自己复习时最需要的答案表达"
            />
          </div>

          <div class="detail-surface">
            <div class="mb-3 flex items-center gap-2 text-slate-500">
              <Sparkles class="h-4 w-4 text-amber-500" />
              <p class="text-xs font-black uppercase tracking-[0.14em]">关键解析</p>
            </div>
            <textarea
              v-model="form.analysis"
              class="detail-textarea min-h-32 text-sm leading-6 text-slate-700"
              placeholder="把解题思路、公式依据、容易漏掉的判断放在这里"
            />
          </div>
        </div>
      </section>

      <section v-if="draft" class="soft-panel overflow-hidden p-0">
        <div class="detail-band detail-band-mistake p-4">
          <div class="flex items-center gap-3">
            <div class="icon-well bg-white/20 text-white">
              <Brain class="h-4 w-4" />
            </div>
            <div>
              <p class="text-xs font-black uppercase tracking-[0.16em] text-white/74">纠错重点</p>
              <h2 class="text-lg font-black text-white">把错误和提醒拎清楚</h2>
            </div>
          </div>
        </div>

        <div class="grid gap-4 p-4">
          <div v-if="item?.wrongAnswerText" class="detail-surface">
            <p class="mb-3 text-xs font-black uppercase tracking-[0.14em] text-rose-500">识别到的错误答案</p>
            <p class="whitespace-pre-wrap text-sm leading-6 text-ink">{{ item.wrongAnswerText }}</p>
          </div>

          <div class="detail-surface">
            <p class="mb-3 text-xs font-black uppercase tracking-[0.14em] text-rose-500">错因分析</p>
            <textarea
              v-model="form.mistakeAnalysis"
              class="detail-textarea min-h-26 text-sm leading-6 text-slate-700"
              placeholder="说明这题为什么会错：概念不清、公式乱用、条件漏看、计算粗心..."
            />
          </div>

          <div class="detail-surface">
            <div class="mb-3 flex items-center gap-2 text-slate-500">
              <NotebookPen class="h-4 w-4 text-indigo-500" />
              <p class="text-xs font-black uppercase tracking-[0.14em]">我的提醒</p>
            </div>
            <textarea
              v-model="form.userNotes"
              class="detail-textarea min-h-24 text-sm leading-6 text-slate-700"
              placeholder="给下次复习的自己留一句提醒：先看定义、先列已知、先验算哪一步"
            />
          </div>
        </div>
      </section>

      <section v-if="draft" class="soft-panel p-4">
        <div class="flex items-center gap-2 text-slate-500">
          <Tag class="h-4 w-4 text-blue-500" />
          <p class="text-xs font-black uppercase tracking-[0.14em]">知识点锚点</p>
        </div>
        <input
          v-model="form.knowledgePointsText"
          class="input-soft mt-3 h-12 w-full px-4 text-sm text-ink"
          placeholder="用 、 或逗号分隔，例如：相似三角形、分类讨论"
        />
        <div v-if="knowledgePoints.length" class="mt-3 flex flex-wrap gap-2">
          <span v-for="point in knowledgePoints" :key="point" class="detail-chip">
            {{ point }}
          </span>
        </div>

        <button class="green-gradient mt-4 flex h-12 w-full items-center justify-center gap-2 rounded-2xl text-sm font-bold text-white" :disabled="busy" @click="save">
          <Check class="h-4 w-4" />
          {{ busy ? '保存中...' : '保存这张错题卡' }}
        </button>
      </section>
    </main>
  </div>
</template>
