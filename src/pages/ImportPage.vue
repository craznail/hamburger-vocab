<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { Upload } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import FileUpload from '../components/FileUpload.vue'
import NavBar from '../components/NavBar.vue'
import fileTxt from '../assets/ui-icons/file-txt.svg'
import filePdf from '../assets/ui-icons/file-pdf.svg'
import fileMd from '../assets/ui-icons/file-md.svg'
import fileDoc from '../assets/ui-icons/file-doc.svg'

const router = useRouter()
const store = useAppStore()
const importResult = ref(null)
const fileIcons = [fileTxt, filePdf, fileMd, fileDoc]

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
        <h1 class="page-header-title text-[1.7rem]">导入</h1>
      </template>
    </NavBar>

    <main class="flex-1 px-4 pb-8 pt-4">
      <section class="soft-panel p-4">
        <div class="rounded-[28px] border-2 border-dashed border-blue-200 bg-blue-50/40 px-4 py-8 text-center">
          <div class="mb-5 flex justify-center gap-4">
            <img v-for="icon in fileIcons" :key="icon" :src="icon" alt="文件类型" class="h-11 w-11 rounded-xl bg-white shadow-sm" />
          </div>
          <FileUpload @file-selected="onFileSelected" />
        </div>
      </section>

      <section class="mt-6">
        <div class="mb-3 flex items-center justify-between">
          <h2 class="text-sm font-black text-ink">导入历史</h2>
          <button class="text-xs font-bold text-slate-400">查看全部</button>
        </div>
        <div class="card-list-row flex items-center gap-3 p-4">
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
