<script setup>
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { ArrowRight, Cloud, FileClock, Heart, Settings, SlidersHorizontal, Star, UserRound, Volume2 } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import BottomNav from '../components/BottomNav.vue'
import NavBar from '../components/NavBar.vue'

const router = useRouter()
const store = useAppStore()

const totalCards = computed(() => store.decks.reduce((sum, deck) => sum + (deck.wordCount || deck.word_count || deck.total || 0), 0))
const masteredCards = computed(() => store.learningStats?.masteredCards || 0)
const streakDays = computed(() => store.learningStats?.streakDays || 0)
const studyRows = [
  { label: '学习统计', hint: '复习时长、趋势和掌握分布', icon: FileClock, route: 'Stats' },
  { label: '学习成就', hint: '连续学习和累计里程碑', icon: Star, route: 'Achievements' }
]
const settingRows = [
  { label: '同步与服务端', hint: '登录 wrong-notebook 后同步错题', icon: Cloud, route: 'Settings' },
  { label: '学习设置', hint: '复习节奏和卡片偏好', icon: SlidersHorizontal, route: 'Settings' },
  { label: 'TTS 发音设置', hint: '调整朗读服务和语音', icon: Volume2, route: 'Settings' },
  { label: '关于应用', hint: '版本与本地数据说明', icon: Settings, route: 'Settings' }
]
</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar :showBack="false">
      <template #left>
        <div>
          <h1 class="page-header-title text-[1.85rem]">我的</h1>
          <p class="page-subtitle mt-1">本地学习档案</p>
        </div>
      </template>
    </NavBar>

    <main class="page-shell">
      <section class="glass-card overflow-hidden px-4 py-5">
        <div class="flex items-center gap-4">
          <div class="grid h-16 w-16 place-items-center rounded-full bg-[linear-gradient(180deg,#eef4ff_0%,#dce8ff_100%)] text-blue-500 shadow-[inset_0_1px_0_rgba(255,255,255,0.9)]">
            <UserRound class="h-9 w-9" />
          </div>
          <div class="min-w-0 flex-1">
            <h2 class="text-lg font-black text-ink">学习者</h2>
            <p class="mt-1 text-xs font-semibold text-slate-400">已连续学习 {{ streakDays }} 天</p>
          </div>
          <span class="rounded-full bg-[#eef9f6] px-3 py-1.5 text-xs font-black text-[#249b82]">本地档案</span>
        </div>
      </section>

      <section class="mt-5 grid grid-cols-3 gap-3">
        <div class="stat-grid-card p-3 text-center">
          <div class="text-xl font-black text-ink">{{ store.decks.length }}</div>
          <p class="mt-1 text-[11px] text-slate-400">知识库</p>
        </div>
        <div class="stat-grid-card p-3 text-center">
          <div class="text-xl font-black text-ink">{{ masteredCards }}</div>
          <p class="mt-1 text-[11px] text-slate-400">已掌握</p>
        </div>
        <div class="stat-grid-card p-3 text-center">
          <div class="text-xl font-black text-ink">{{ totalCards }}</div>
          <p class="mt-1 text-[11px] text-slate-400">总卡片</p>
        </div>
      </section>

      <section class="mt-5">
        <div class="section-title-row">
          <h2 class="section-title">学习记录</h2>
        </div>
        <div class="soft-panel overflow-hidden">
          <button v-for="row in studyRows" :key="row.label" class="flex w-full items-center gap-3 border-b border-blue-50 px-4 py-3.5 text-left last:border-b-0" @click="row.route && router.push({ name: row.route })">
            <span class="grid h-10 w-10 place-items-center rounded-[15px] bg-blue-50 text-blue-500">
              <component :is="row.icon || Heart" class="h-4.5 w-4.5" />
            </span>
            <span class="min-w-0 flex-1">
              <span class="block text-sm font-black text-ink">{{ row.label }}</span>
              <span class="mt-1 block truncate text-xs text-slate-400">{{ row.hint }}</span>
            </span>
            <ArrowRight class="h-4 w-4 text-slate-300" />
          </button>
        </div>
      </section>

      <section class="mt-5">
        <div class="section-title-row">
          <h2 class="section-title">数据与设置</h2>
        </div>
        <div class="soft-panel overflow-hidden">
          <button v-for="row in settingRows" :key="row.label" class="flex w-full items-center gap-3 border-b border-blue-50 px-4 py-3.5 text-left last:border-b-0" @click="row.route && router.push({ name: row.route })">
          <span class="grid h-10 w-10 place-items-center rounded-[15px] bg-[#f4f7ff] text-blue-500">
            <component :is="row.icon || Heart" class="h-4 w-4" />
          </span>
          <span class="min-w-0 flex-1">
            <span class="block text-sm font-black text-ink">{{ row.label }}</span>
            <span class="mt-1 block truncate text-xs text-slate-400">{{ row.hint }}</span>
          </span>
          <ArrowRight class="h-4 w-4 text-slate-300" />
        </button>
        </div>
      </section>

      <p class="mt-5 px-1 text-xs leading-5 text-slate-400">数据保存在当前设备，可按需同步到服务端。</p>
    </main>

    <BottomNav />
  </div>
</template>
