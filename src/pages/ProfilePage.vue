<script setup>
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowRight, Crown, Download, Edit3, FileClock, Heart, Recycle, Settings, Star, UserRound } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import BottomNav from '../components/BottomNav.vue'
import NavBar from '../components/NavBar.vue'

const router = useRouter()
const store = useAppStore()

const totalCards = computed(() => store.decks.reduce((sum, deck) => sum + (deck.wordCount || deck.word_count || deck.total || 0), 0))
const rows = [
  { label: '学习记录', icon: FileClock, route: 'Stats' },
  { label: '我的收藏', icon: Star, route: 'Achievements' },
  { label: '回收站', icon: Recycle },
  { label: '导入记录', icon: Download, route: 'Import' },
  { label: '设置', icon: Settings, route: 'Settings' }
]
</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar :showBack="false">
      <template #left>
        <h1 class="text-xl font-black text-ink">我的</h1>
      </template>
    </NavBar>

    <main class="flex-1 px-5 pb-8">
      <section class="flex items-center gap-4">
        <div class="grid h-16 w-16 place-items-center rounded-full bg-blue-100 text-blue-500">
          <UserRound class="h-9 w-9" />
        </div>
        <div class="min-w-0 flex-1">
          <h2 class="text-lg font-black text-ink">记得住</h2>
          <p class="mt-1 text-xs text-slate-400">hello@recall.com</p>
        </div>
        <button class="flex items-center gap-1 rounded-lg bg-blue-50 px-3 py-2 text-xs font-bold text-blue-600">
          <Edit3 class="h-3.5 w-3.5" />
          编辑资料
        </button>
      </section>

      <section class="mt-5 flex items-center gap-3 rounded-2xl bg-amber-50 p-4 text-amber-700">
        <Crown class="h-7 w-7 shrink-0 text-amber-500" />
        <div class="min-w-0 flex-1">
          <p class="text-sm font-black">Pro 会员</p>
          <p class="mt-1 text-xs text-amber-600/75">2025-06-18 到期</p>
        </div>
        <button class="flex items-center gap-1 text-xs font-bold">
          去续费
          <ArrowRight class="h-3.5 w-3.5" />
        </button>
      </section>

      <section class="mt-5 grid grid-cols-3 gap-3">
        <div class="tiny-card rounded-2xl p-3 text-center">
          <div class="text-xl font-black text-ink">{{ store.decks.length }}</div>
          <p class="mt-1 text-[11px] text-slate-400">知识库</p>
        </div>
        <div class="tiny-card rounded-2xl p-3 text-center">
          <div class="text-xl font-black text-ink">{{ totalCards }}</div>
          <p class="mt-1 text-[11px] text-slate-400">总卡片</p>
        </div>
        <div class="tiny-card rounded-2xl p-3 text-center">
          <div class="text-xl font-black text-ink">12</div>
          <p class="mt-1 text-[11px] text-slate-400">连续天数</p>
        </div>
      </section>

      <section class="soft-panel mt-5 overflow-hidden rounded-2xl">
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
