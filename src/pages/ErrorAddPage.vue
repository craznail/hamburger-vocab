<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'
import { Check, ImagePlus, Loader } from 'lucide-vue-next'
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
  knowledgePointsText: '',
})

const imageSrc = computed(() => draft.value?.localImagePath ? convertFileSrc(draft.value.localImagePath) : '')

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
    form.value.knowledgePointsText = errorApi.parseKnowledgePoints(item.value.knowledgePoints).join('、')
    status.value = 'AI 分析完成，可编辑后保存'
  } catch (e) {
    status.value = e instanceof Error ? e.message : String(e)
  } finally {
    busy.value = false
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
      knowledgePoints: form.value.knowledgePointsText.split(/[、,\n]/).map(x => x.trim()).filter(Boolean),
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
          <p class="mt-1 text-xs text-slate-400">先保存本地草稿，再调用服务端 AI</p>
        </div>
      </template>
    </NavBar>

    <main class="px-4 pt-4">
      <section class="soft-panel mb-4 p-5 text-center">
        <label class="inline-flex h-12 items-center justify-center gap-2 rounded-2xl bg-blue-600 px-5 text-sm font-bold text-white">
          <ImagePlus class="h-4 w-4" />
          选择图片
          <input class="hidden" type="file" accept="image/*" @change="chooseImage" />
        </label>
        <p v-if="status" class="mt-3 text-xs font-semibold text-slate-500">{{ status }}</p>
        <Loader v-if="busy" class="mx-auto mt-3 h-5 w-5 animate-spin text-blue-500" />
      </section>

      <img v-if="imageSrc" :src="imageSrc" class="mb-4 max-h-64 w-full rounded-3xl object-contain bg-white shadow" />

      <section v-if="draft" class="soft-panel grid gap-3 p-4">
        <textarea v-model="form.questionText" class="input-soft min-h-24 p-3 text-sm" placeholder="题干" />
        <textarea v-model="form.answerText" class="input-soft min-h-20 p-3 text-sm" placeholder="答案" />
        <textarea v-model="form.analysis" class="input-soft min-h-28 p-3 text-sm" placeholder="解析" />
        <textarea v-model="form.mistakeAnalysis" class="input-soft min-h-20 p-3 text-sm" placeholder="错因分析" />
        <input v-model="form.knowledgePointsText" class="input-soft h-11 px-3 text-sm" placeholder="知识点，用顿号或逗号分隔" />
        <button class="green-gradient flex h-12 items-center justify-center gap-2 rounded-2xl text-sm font-bold text-white" :disabled="busy" @click="save">
          <Check class="h-4 w-4" />
          保存错题
        </button>
      </section>
    </main>
  </div>
</template>
