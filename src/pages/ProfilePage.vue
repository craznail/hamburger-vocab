<script setup>
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowRight, Download, FileClock, Heart, Settings, Star, UserRound } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import BottomNav from '../components/BottomNav.vue'
import NavBar from '../components/NavBar.vue'

const router = useRouter()
const store = useAppStore()

const totalCards = computed(() => store.decks.reduce((sum, deck) => sum + (deck.wordCount || deck.word_count || deck.total || 0), 0))
const streakDays = computed(() => store.learningStats?.streakDays || 0)
const rows = [
  { label: '学习记录', icon: FileClock, route: 'Stats' },
  { label: '学习成就', icon: Star, route: 'Achievements' },
  { label: '导入记录', icon: Download, route: 'Import' },
  { label: '设置', icon: Settings, route: 'Settings' }
]
</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar :showBack="false">
      <template #left>
        <h1 class="page-header-title text-[1.85rem]">我的</h1>
      </template>
    </NavBar>

    <main class="flex-1 px-4 pb-8 pt-4">
      <section class="glass-card flex items-center gap-4 px-4 py-4">
        <div class="grid h-16 w-16 place-items-center rounded-full bg-[linear-gradient(180deg,#eff4ff_0%,#dfeafe_100%)] text-blue-500">
          <UserRound class="h-9 w-9" />
        </div>
        <div class="min-w-0 flex-1">
          <h2 class="text-lg font-black text-ink">本地学习档案</h2>
          <p class="mt-1 text-xs text-slate-400">学习数据保存在当前设备</p>
        </div>
      </section>

      <section class="mt-5 grid grid-cols-3 gap-3">
        <div class="stat-grid-card p-3 text-center">
          <div class="text-xl font-black text-ink">{{ store.decks.length }}</div>
          <p class="mt-1 text-[11px] text-slate-400">知识库</p>
        </div>
        <div class="stat-grid-card p-3 text-center">
          <div class="text-xl font-black text-ink">{{ totalCards }}</div>
          <p class="mt-1 text-[11px] text-slate-400">总卡片</p>
        </div>
        <div class="stat-grid-card p-3 text-center">
          <div class="text-xl font-black text-ink">{{ streakDays }}</div>
          <p class="mt-1 text-[11px] text-slate-400">连续天数</p>
        </div>
      </section>

      <section class="soft-panel mt-5 overflow-hidden">
        <button v-for="row in rows" :key="row.label" class="flex w-full items-center gap-3 border-b border-blue-50 px-4 py-3 text-left last:border-b-0" @click="row.route && router.push({ name: row.route })">
          <span class="grid h-8 w-8 place-items-center rounded-lg bg-blue-50 text-blue-500">
            <component :is="row.icon || Heart" class="h-4 w-4" />
          </span>
          <span class="flex-1 text-sm font-bold text-ink">{{ row.label }}</span>
          <ArrowRight class="h-4 w-4 text-slate-300" />
        </button>
      </section>
    </main>

    <BottomNav />
  </div>
</template>
