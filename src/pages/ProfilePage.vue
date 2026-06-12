<script setup>
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowRight, Crown, Edit3, UserRound } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import BottomNav from '../components/BottomNav.vue'
import NavBar from '../components/NavBar.vue'
import rowHistory from '../assets/ui-icons/row-history.svg'
import rowFavorite from '../assets/ui-icons/row-favorite.svg'
import rowTrash from '../assets/ui-icons/row-trash.svg'
import rowImport from '../assets/ui-icons/row-import.svg'
import rowSettings from '../assets/ui-icons/row-settings.svg'
import profileAvatar from '../assets/ui-icons/profile-avatar.svg'
import proBadge from '../assets/ui-icons/pro-badge.svg'

const router = useRouter()
const store = useAppStore()

const totalCards = computed(() => store.decks.reduce((sum, deck) => sum + (deck.wordCount || deck.word_count || deck.total || 0), 0))
const rows = [
  { label: '学习记录', icon: rowHistory, route: 'Stats' },
  { label: '我的收藏', icon: rowFavorite, route: 'Achievements' },
  { label: '回收站', icon: rowTrash },
  { label: '导入记录', icon: rowImport, route: 'Import' },
  { label: '设置', icon: rowSettings, route: 'Settings' }
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
        <img :src="profileAvatar" alt="头像" class="h-16 w-16 rounded-full" />
        <div class="min-w-0 flex-1">
          <h2 class="text-lg font-black text-ink">记得住</h2>
          <p class="mt-1 text-xs text-slate-400">hello@recall.com</p>
        </div>
        <button class="ghost-button px-3 py-2 text-xs font-bold text-blue-600">
          <Edit3 class="h-3.5 w-3.5" />
          编辑资料
        </button>
      </section>

      <section class="mt-5 flex items-center gap-3 rounded-[26px] border border-[#ffe2a8] bg-[linear-gradient(135deg,#fff7df_0%,#fff0bf_100%)] p-4 text-amber-700 shadow-[0_16px_30px_rgba(255,190,68,0.12)]">
        <img :src="proBadge" alt="Pro 会员" class="h-12 w-12 shrink-0" />
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
        <div class="stat-grid-card p-3 text-center">
          <div class="text-xl font-black text-ink">{{ store.decks.length }}</div>
          <p class="mt-1 text-[11px] text-slate-400">知识库</p>
        </div>
        <div class="stat-grid-card p-3 text-center">
          <div class="text-xl font-black text-ink">{{ totalCards }}</div>
          <p class="mt-1 text-[11px] text-slate-400">总卡片</p>
        </div>
        <div class="stat-grid-card p-3 text-center">
          <div class="text-xl font-black text-ink">12</div>
          <p class="mt-1 text-[11px] text-slate-400">连续天数</p>
        </div>
      </section>

      <section class="soft-panel mt-5 overflow-hidden">
        <button v-for="row in rows" :key="row.label" class="flex w-full items-center gap-3 border-b border-blue-50 px-4 py-3 text-left last:border-b-0" @click="row.route && router.push({ name: row.route })">
          <img :src="row.icon" :alt="row.label" class="h-8 w-8 rounded-lg" />
          <span class="flex-1 text-sm font-bold text-ink">{{ row.label }}</span>
          <ArrowRight class="h-4 w-4 text-slate-300" />
        </button>
      </section>
    </main>

    <BottomNav />
  </div>
</template>
