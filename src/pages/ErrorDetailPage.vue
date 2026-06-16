<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'
import { Save } from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
import * as errorApi from '../api/errorItem'

const route = useRoute()
const router = useRouter()
const item = ref<errorApi.ErrorItem | null>(null)
const form = ref({
  questionText: '',
  answerText: '',
  analysis: '',
  mistakeAnalysis: '',
  userNotes: '',
  knowledgePointsText: '',
})

const imageSrc = computed(() => item.value?.localImagePath ? convertFileSrc(item.value.localImagePath) : item.value?.remoteImageUrl || '')

onMounted(load)

async function load() {
  const items = await errorApi.getErrorItems()
  item.value = items.find(x => x.id === route.params.id) || null
  if (item.value) {
    form.value.questionText = item.value.questionText || ''
    form.value.answerText = item.value.answerText || ''
    form.value.analysis = item.value.analysis || ''
    form.value.mistakeAnalysis = item.value.mistakeAnalysis || ''
    form.value.userNotes = item.value.userNotes || ''
    form.value.knowledgePointsText = errorApi.parseKnowledgePoints(item.value.knowledgePoints).join('、')
  }
}

async function save() {
  if (!item.value) return
  await errorApi.saveErrorItem({
    id: item.value.id,
    questionText: form.value.questionText,
    answerText: form.value.answerText,
    analysis: form.value.analysis,
    mistakeAnalysis: form.value.mistakeAnalysis,
    userNotes: form.value.userNotes,
    knowledgePoints: form.value.knowledgePointsText.split(/[、,\n]/).map(x => x.trim()).filter(Boolean),
  })
  await load()
}
</script>

<template>
  <div class="app-page min-h-screen">
    <NavBar @back="router.push({ name: 'ErrorNotebook' })">
      <template #left>
        <div>
          <h1 class="text-sm font-black text-ink">错题详情</h1>
          <p class="mt-1 text-xs text-slate-400">{{ item?.syncStatus || '' }}</p>
        </div>
      </template>
    </NavBar>

    <main v-if="item" class="grid gap-4 px-4 pt-4">
      <img v-if="imageSrc" :src="imageSrc" class="max-h-72 w-full rounded-3xl object-contain bg-white shadow" />
      <section class="soft-panel grid gap-3 p-4">
        <textarea v-model="form.questionText" class="input-soft min-h-24 p-3 text-sm" placeholder="题干" />
        <textarea v-model="form.answerText" class="input-soft min-h-20 p-3 text-sm" placeholder="答案" />
        <textarea v-model="form.analysis" class="input-soft min-h-28 p-3 text-sm" placeholder="解析" />
        <textarea v-model="form.mistakeAnalysis" class="input-soft min-h-20 p-3 text-sm" placeholder="错因分析" />
        <textarea v-model="form.userNotes" class="input-soft min-h-20 p-3 text-sm" placeholder="我的笔记" />
        <input v-model="form.knowledgePointsText" class="input-soft h-11 px-3 text-sm" placeholder="知识点" />
        <button class="blue-gradient flex h-12 items-center justify-center gap-2 rounded-2xl text-sm font-bold text-white" @click="save">
          <Save class="h-4 w-4" />
          保存修改
        </button>
      </section>
    </main>
  </div>
</template>
