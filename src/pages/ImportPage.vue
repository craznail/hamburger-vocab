<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { FileCode2, FileText, Upload } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import FileUpload from '../components/FileUpload.vue'
import NavBar from '../components/NavBar.vue'

const router = useRouter()
const store = useAppStore()
const importResult = ref(null)

async function onFileSelected(fileName, text) {
  const result = await store.importFile(fileName, text)
  importResult.value = result
  if (result.success) {
    router.push({ name: 'Study', query: { deckId: result.deckId } })
  } else {
    alert(result.error)
  }
}
</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar @back="router.push({ name: 'Library' })">
      <template #left>
        <h1 class="text-xl font-black text-ink">导入</h1>
      </template>
    </NavBar>

    <main class="flex-1 px-5 pb-8">
      <section class="soft-panel rounded-2xl p-4">
        <div class="rounded-2xl border-2 border-dashed border-blue-200 bg-blue-50/40 px-4 py-8 text-center">
          <div class="mb-5 flex justify-center gap-4">
            <span class="grid h-11 w-11 place-items-center rounded-xl bg-white text-blue-500 shadow-sm"><FileText class="h-5 w-5" /></span>
            <span class="grid h-11 w-11 place-items-center rounded-xl bg-white text-emerald-500 shadow-sm"><FileCode2 class="h-5 w-5" /></span>
          </div>
          <FileUpload @file-selected="onFileSelected" />
        </div>
      </section>

      <section class="mt-6">
        <div class="mb-3 flex items-center justify-between">
          <h2 class="text-sm font-black text-ink">导入历史</h2>
          <button class="text-xs font-bold text-slate-400">查看全部</button>
        </div>
        <div class="soft-panel flex items-center gap-3 rounded-2xl p-4">
          <span class="grid h-11 w-11 place-items-center rounded-xl bg-blue-50 text-blue-500"><Upload class="h-5 w-5" /></span>
          <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-black text-ink">{{ importResult?.deckName || 'Redis 笔记.txt' }}</p>
            <p class="mt-1 text-xs text-slate-400">{{ importResult?.success ? `成功导入 ${importResult.count} 张卡片` : '成功导入 382 张卡片' }}</p>
          </div>
          <span class="text-xs text-slate-400">今天 14:30</span>
        </div>
      </section>
    </main>
  </div>
</template>
