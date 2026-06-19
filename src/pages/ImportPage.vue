<script setup>
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { FileCode2, FileText, Upload } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import FileUpload from '../components/FileUpload.vue'
import NavBar from '../components/NavBar.vue'

const router = useRouter()
const store = useAppStore()
const importResult = ref(null)
const importHistory = computed(() => [...store.decks]
  .sort((a, b) => new Date(b.createdAt || b.created_at) - new Date(a.createdAt || a.created_at)))

function deckCount(deck) {
  return deck.wordCount || deck.word_count || 0
}

function formatImportedAt(value) {
  if (!value) return ''
  const date = new Date(value.replace(' ', 'T'))
  return new Intl.DateTimeFormat('zh-CN', {
    month: 'numeric',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(date)
}

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
        <h1 class="page-header-title">导入</h1>
      </template>
    </NavBar>

    <main class="flex-1 px-4 pb-8 pt-4">
      <section class="soft-panel p-4">
        <div class="rounded-[28px] border-2 border-dashed border-blue-200 bg-blue-50/40 px-4 py-8 text-center">
          <div class="mb-5 flex justify-center gap-4">
            <span class="grid h-11 w-11 place-items-center rounded-xl bg-white text-blue-500 shadow-sm"><FileText class="h-5 w-5" /></span>
            <span class="grid h-11 w-11 place-items-center rounded-xl bg-white text-emerald-500 shadow-sm"><FileCode2 class="h-5 w-5" /></span>
          </div>
          <FileUpload @file-selected="onFileSelected" />
        </div>

        <div class="mt-4 rounded-[22px] bg-white/80 p-4 text-left shadow-sm ring-1 ring-blue-100">
          <div class="flex items-center justify-between gap-3">
            <h2 class="text-sm font-black text-ink">导入格式</h2>
            <span class="rounded-full bg-blue-50 px-2.5 py-1 text-[11px] font-bold text-blue-500">仅支持此格式</span>
          </div>
          <p class="mt-2 text-xs leading-5 text-slate-500">
            每条数据首行是单词，中间各行是词形变化，最后一行是释义。两条数据之间保留一个空行。
          </p>
          <pre class="mt-3 overflow-x-auto rounded-2xl bg-slate-950 px-4 py-3 text-[12px] leading-6 text-slate-100"><code>am
was been
是

is
was been
是

are
were been
是

can
could could
能</code></pre>
        </div>
      </section>

      <section class="mt-6">
        <div class="mb-3 flex items-center justify-between">
          <h2 class="text-sm font-black text-ink">导入历史</h2>
          <button class="text-xs font-bold text-slate-400">查看全部</button>
        </div>
        <button
          v-for="deck in importHistory"
          :key="deck.id"
          class="card-list-row mb-3 flex w-full items-center gap-3 p-4 text-left"
          @click="router.push({ name: 'DeckDetail', params: { id: deck.id } })"
        >
          <span class="grid h-11 w-11 place-items-center rounded-xl bg-blue-50 text-blue-500"><Upload class="h-5 w-5" /></span>
          <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-black text-ink">{{ deck.name }}</p>
            <p class="mt-1 text-xs text-slate-400">已导入 {{ deckCount(deck) }} 张卡片</p>
          </div>
          <span class="text-xs text-slate-400">{{ formatImportedAt(deck.createdAt || deck.created_at) }}</span>
        </button>
        <div v-if="importHistory.length === 0" class="soft-panel p-6 text-center text-sm text-slate-400">
          暂无导入记录
        </div>
      </section>
    </main>
  </div>
</template>
