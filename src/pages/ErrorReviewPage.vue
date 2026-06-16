<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'
import { CheckCircle, RotateCcw } from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
import * as errorApi from '../api/errorItem'

const router = useRouter()
const items = ref<errorApi.ErrorItem[]>([])
const index = ref(0)
const revealed = ref(false)
const startedAt = ref(Date.now())
const done = ref(false)

const current = computed(() => items.value[index.value] || null)
const imageSrc = computed(() => current.value?.localImagePath ? convertFileSrc(current.value.localImagePath) : current.value?.remoteImageUrl || '')

onMounted(async () => {
  items.value = await errorApi.getDueErrorItems()
  if (items.value.length === 0) done.value = true
})

async function rate(quality: number) {
  if (!current.value) return
  const seconds = Math.max(1, Math.round((Date.now() - startedAt.value) / 1000))
  await errorApi.rateErrorItem(current.value.id, quality, seconds)
  if (index.value < items.value.length - 1) {
    index.value++
    revealed.value = false
    startedAt.value = Date.now()
  } else {
    done.value = true
  }
}
</script>

<template>
  <div class="app-page min-h-screen">
    <NavBar @back="router.push({ name: 'ErrorNotebook' })">
      <template #left>
        <div>
          <h1 class="text-sm font-black text-ink">错题复习</h1>
          <p class="mt-1 text-xs text-slate-400">{{ done ? '完成' : `${index + 1} / ${items.length}` }}</p>
        </div>
      </template>
    </NavBar>

    <main class="flex min-h-[70vh] items-center px-4 pt-4">
      <section v-if="done" class="soft-panel w-full p-8 text-center">
        <CheckCircle class="mx-auto mb-4 h-16 w-16 text-green-400" />
        <h2 class="mb-2 text-xl font-black text-ink">错题复习完成</h2>
        <p class="mb-6 text-sm text-slate-400">本次复习 {{ items.length }} 道错题</p>
        <button class="blue-gradient h-11 rounded-xl px-6 text-sm font-bold text-white" @click="router.push({ name: 'ErrorNotebook' })">返回错题本</button>
      </section>

      <section v-else-if="current" class="soft-panel w-full p-4">
        <img v-if="imageSrc" :src="imageSrc" class="mb-4 max-h-56 w-full rounded-2xl object-contain bg-white" />
        <h2 class="whitespace-pre-wrap text-base font-black leading-7 text-ink">{{ current.questionText || '这道题还没有题干，先根据图片回忆。' }}</h2>

        <button v-if="!revealed" class="blue-gradient mt-6 flex h-12 w-full items-center justify-center gap-2 rounded-2xl text-sm font-bold text-white" @click="revealed = true">
          <RotateCcw class="h-4 w-4" />
          查看答案
        </button>

        <div v-else class="mt-5 grid gap-4">
          <div class="rounded-2xl bg-blue-50 p-4">
            <p class="mb-2 text-xs font-black text-blue-500">答案</p>
            <p class="whitespace-pre-wrap text-sm leading-6 text-ink">{{ current.answerText || '暂无答案' }}</p>
          </div>
          <div class="rounded-2xl bg-slate-50 p-4">
            <p class="mb-2 text-xs font-black text-slate-500">解析</p>
            <p class="whitespace-pre-wrap text-sm leading-6 text-slate-600">{{ current.analysis || current.mistakeAnalysis || '暂无解析' }}</p>
          </div>
          <div class="grid grid-cols-3 gap-2">
            <button class="red-gradient h-12 rounded-2xl text-sm font-black text-white" @click="rate(0)">忘了</button>
            <button class="warm-gradient h-12 rounded-2xl text-sm font-black text-white" @click="rate(3)">模糊</button>
            <button class="green-gradient h-12 rounded-2xl text-sm font-black text-white" @click="rate(5)">掌握</button>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>
