<script setup>
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowRight, BookOpen, Box, Code2, Globe2, Languages, Plus, Search } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import BottomNav from '../components/BottomNav.vue'
import NavBar from '../components/NavBar.vue'

const router = useRouter()
const store = useAppStore()

const deckIcons = [Box, Languages, Code2, Globe2, BookOpen]
const deckColors = [
  'from-red-500 to-orange-400',
  'from-emerald-500 to-green-400',
  'from-blue-500 to-cyan-400',
  'from-purple-500 to-fuchsia-400',
  'from-amber-500 to-yellow-400'
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
        <h1 class="text-xl font-black text-ink">我的知识库</h1>
      </template>
      <template #right>
        <div class="flex items-center gap-2">
          <button class="flex h-10 w-10 items-center justify-center rounded-full bg-blue-50 text-blue-600" title="导入" @click="router.push({ name: 'Import' })">
            <Plus class="h-5 w-5" />
          </button>
          <button class="flex h-10 w-10 items-center justify-center rounded-full bg-white text-slate-400 shadow-sm" title="搜索">
            <Search class="h-5 w-5" />
          </button>
        </div>
      </template>
    </NavBar>

    <main class="flex-1 px-5 pb-8">
      <section class="grid grid-cols-3 gap-3">
        <div class="tiny-card rounded-2xl p-4 text-center">
          <div class="text-2xl font-black text-blue-600">{{ totals.decks }}</div>
          <div class="mt-1 text-[11px] font-medium text-slate-400">知识库总数</div>
        </div>
        <div class="tiny-card rounded-2xl p-4 text-center">
          <div class="text-2xl font-black text-ink">{{ totals.totalCards.toLocaleString() }}</div>
          <div class="mt-1 text-[11px] font-medium text-slate-400">总卡片数</div>
        </div>
        <div class="tiny-card rounded-2xl p-4 text-center">
          <div class="text-2xl font-black text-ink">{{ totals.rate }}%</div>
          <div class="mt-1 text-[11px] font-medium text-slate-400">平均掌握率</div>
        </div>
      </section>

      <section class="mt-5 space-y-3">
        <button
          v-for="(deck, index) in store.sortedDecks"
          :key="deck.id"
          class="soft-panel flex w-full items-center gap-3 rounded-2xl px-4 py-3 text-left active:scale-[0.99]"
          @click="openDeck(deck.id)"
        >
          <span class="deck-gem flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br text-white shadow-lg shadow-blue-100" :class="deckColors[index % deckColors.length]">
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

        <div v-if="store.decks.length === 0" class="soft-panel rounded-2xl p-8 text-center">
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
