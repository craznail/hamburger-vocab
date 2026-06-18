<script setup>
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowRight, BookOpen, Box, Code2, Globe2, Languages, Plus, Search } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import BottomNav from '../components/BottomNav.vue'
import NavBar from '../components/NavBar.vue'

const router = useRouter()
const store = useAppStore()

onMounted(() => {
  store.refreshAll()
})

const deckIcons = [Box, Languages, Code2, Globe2, BookOpen]
const deckColors = [
  'from-blue-500 to-cyan-400',
  'from-emerald-500 to-teal-400',
  'from-amber-500 to-yellow-400',
  'from-sky-500 to-blue-400',
  'from-rose-400 to-orange-300'
]

const totals = computed(() => {
  const totalCards = store.decks.reduce((sum, deck) => sum + getDeckTotal(deck), 0)
  const mastered = store.decks.reduce((sum, deck) => sum + getDeckMastered(deck), 0)
  const rate = totalCards ? Math.round((mastered / totalCards) * 100) : 0
  return { decks: store.decks.length, totalCards, rate }
})

function getDeckTotal(deck) {
  return deck.wordCount || deck.word_count || deck.total || 0
}

function getDeckMastered(deck) {
  return deck.masteredCount || deck.mastered_count || deck.mastered || 0
}

function getDeckDue(deck) {
  return deck.dueCount || deck.due_count || deck.due || 0
}

function getRatio(deck) {
  const total = getDeckTotal(deck)
  if (!total) return 0
  return Math.round((getDeckMastered(deck) / total) * 100)
}

function openDeck(deckId) {
  router.push({ name: 'DeckDetail', params: { id: deckId } })
}
</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar :showBack="false">
      <template #left>
        <div>
          <h1 class="page-header-title text-[1.9rem]">我的知识库</h1>
          <p class="page-subtitle mt-1">把词库整理成可复习的路径</p>
        </div>
      </template>
      <template #right>
        <div class="flex items-center gap-2">
          <button class="grid h-10 w-10 place-items-center rounded-full border border-[#d9e5ff] bg-white text-blue-600 shadow-[0_10px_20px_rgba(95,126,194,0.08)]" title="导入" @click="router.push({ name: 'Import' })">
            <Plus class="h-5 w-5" />
          </button>
          <button class="grid h-10 w-10 place-items-center rounded-full border border-[#d9e5ff] bg-white text-slate-400 shadow-[0_10px_20px_rgba(95,126,194,0.08)]" title="搜索">
            <Search class="h-5 w-5" />
          </button>
        </div>
      </template>
    </NavBar>

    <main class="page-shell">
      <section class="glass-card mb-4 overflow-hidden p-5">
        <div class="flex items-start justify-between gap-4">
          <div>
            <p class="section-kicker">知识库总数</p>
            <div class="mt-3 flex items-end gap-2">
              <span class="text-6xl font-black leading-none text-ink">{{ totals.decks }}</span>
              <span class="pb-2 text-sm font-bold text-slate-400">个集合</span>
            </div>
          </div>
          <div class="grid gap-2 text-right">
            <span class="rounded-full bg-[#e9f8f4] px-3 py-1.5 text-xs font-black text-[#2fa88f]">掌握 {{ totals.rate }}%</span>
            <span class="rounded-full bg-[#fff6df] px-3 py-1.5 text-xs font-black text-[#d58b22]">{{ totals.totalCards.toLocaleString() }} 卡片</span>
          </div>
        </div>
        <div class="mt-5 flex gap-2">
          <div class="input-soft flex h-12 flex-1 items-center gap-2 px-3 text-sm font-semibold text-slate-400">
            <Search class="h-4 w-4" />
            搜索词库或单词
          </div>
          <button class="secondary-action h-12 w-12 p-0" @click="router.push({ name: 'Import' })">
            <Plus class="h-5 w-5" />
          </button>
        </div>
        <div class="mt-3 grid grid-cols-3 gap-2 rounded-[18px] border border-blue-100/80 bg-[#f3f7ff] p-1">
          <button class="rounded-[14px] bg-white py-2 text-xs font-black text-blue-600 shadow-sm">全部</button>
          <button class="py-2 text-xs font-black text-slate-400">学习中</button>
          <button class="py-2 text-xs font-black text-slate-400">已掌握</button>
        </div>
      </section>

      <section class="mt-5 space-y-3">
        <div class="section-title-row">
          <h2 class="section-title">词库列表</h2>
        </div>
        <button
          v-for="(deck, index) in store.sortedDecks"
          :key="deck.id"
          class="card-list-row flex w-full items-center gap-3 px-4 py-3.5 text-left active:scale-[0.99]"
          @click="openDeck(deck.id)"
        >
          <span class="deck-gem flex h-[3.25rem] w-[3.25rem] shrink-0 items-center justify-center rounded-[18px] bg-gradient-to-br text-white shadow-lg shadow-blue-100" :class="deckColors[index % deckColors.length]">
            <component :is="deckIcons[index % deckIcons.length]" class="h-6 w-6" />
          </span>
          <span class="min-w-0 flex-1">
            <span class="flex items-center justify-between gap-3">
              <span class="truncate text-sm font-black text-ink">{{ deck.name }}</span>
              <ArrowRight class="h-4 w-4 shrink-0 text-slate-300" />
            </span>
            <span class="mt-1 flex items-center gap-2 text-xs text-slate-400">
              <span>{{ getDeckMastered(deck) }} / {{ getDeckTotal(deck) }}</span>
              <span v-if="getDeckDue(deck) > 0">今日待复习 {{ getDeckDue(deck) }}</span>
            </span>
            <span class="mt-2 flex items-center gap-2">
              <span class="progress-track h-1.5 flex-1">
                <span class="progress-fill block" :style="{ width: `${getRatio(deck)}%` }" />
              </span>
              <span class="w-9 text-right text-xs font-bold text-blue-500">{{ getRatio(deck) }}%</span>
            </span>
          </span>
        </button>

        <div v-if="store.decks.length === 0" class="soft-panel p-8 text-center">
          <BookOpen class="mx-auto mb-3 h-12 w-12 text-blue-200" />
          <p class="text-sm font-bold text-slate-500">还没有知识库</p>
          <button class="blue-gradient mt-5 h-11 rounded-xl px-7 text-sm font-bold text-white" @click="router.push({ name: 'Import' })">
            导入文件
          </button>
        </div>
      </section>
    </main>

    <BottomNav />
  </div>
</template>
