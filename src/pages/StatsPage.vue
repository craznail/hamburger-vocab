<script setup>
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '../stores/useAppStore'
import BottomNav from '../components/BottomNav.vue'
import NavBar from '../components/NavBar.vue'
import statTime from '../assets/ui-icons/stat-time.svg'
import statCards from '../assets/ui-icons/stat-cards.svg'
import statTotal from '../assets/ui-icons/stat-total.svg'
import statRate from '../assets/ui-icons/stat-rate.svg'

const router = useRouter()
const store = useAppStore()

const totalCards = computed(() => store.decks.reduce((sum, deck) => sum + (deck.wordCount || deck.word_count || deck.total || 0), 0))
const dueCards = computed(() => store.decks.reduce((sum, deck) => sum + (deck.dueCount || deck.due_count || deck.due || 0), 0))
const masteredCards = computed(() => store.decks.reduce((sum, deck) => sum + (deck.masteredCount || deck.mastered_count || deck.mastered || 0), 0))
const masteryRate = computed(() => totalCards.value ? Math.round((masteredCards.value / totalCards.value) * 100) : 0)

const bars = [0.8, 0.9, 1.2, 1.1, 1.3, 1.0, 0.5]
const labels = ['周一', '周二', '周三', '周四', '周五', '周六', '周日']
</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar :showBack="false">
      <template #left>
        <h1 class="page-header-title text-[1.85rem]">学习统计</h1>
      </template>
      <template #right>
        <button class="ghost-button px-3 py-1.5 text-xs font-bold text-blue-600">本周</button>
      </template>
    </NavBar>

    <main class="flex-1 px-4 pb-8 pt-4">
      <section class="grid grid-cols-2 gap-3">
        <div class="stat-grid-card p-4">
          <img :src="statTime" alt="学习时长" class="mb-2 h-6 w-6" />
          <div class="text-2xl font-black text-ink">6.8 h</div>
          <p class="mt-1 text-xs text-slate-400">学习时长</p>
          <p class="mt-2 text-xs font-bold text-emerald-500">较上周 ↑ 23%</p>
        </div>
        <div class="stat-grid-card p-4">
          <img :src="statCards" alt="复习卡片" class="mb-2 h-6 w-6" />
          <div class="text-2xl font-black text-ink">{{ dueCards || store.todayCount }}</div>
          <p class="mt-1 text-xs text-slate-400">复习卡片</p>
          <p class="mt-2 text-xs font-bold text-emerald-500">较上周 ↑ 18%</p>
        </div>
        <div class="stat-grid-card p-4">
          <img :src="statTotal" alt="总卡片" class="mb-2 h-6 w-6" />
          <div class="text-2xl font-black text-ink">{{ totalCards }}</div>
          <p class="mt-1 text-xs text-slate-400">总卡片</p>
          <p class="mt-2 text-xs font-bold text-emerald-500">持续积累中</p>
        </div>
        <div class="stat-grid-card p-4">
          <img :src="statRate" alt="记忆准确率" class="mb-2 h-6 w-6" />
          <div class="text-2xl font-black text-ink">{{ masteryRate }}%</div>
          <p class="mt-1 text-xs text-slate-400">记忆准确率</p>
          <p class="mt-2 text-xs font-bold text-emerald-500">较上周 ↑ 6%</p>
        </div>
      </section>

      <section class="soft-panel mt-5 p-4">
        <h2 class="mb-5 text-sm font-black text-ink">学习时长趋势（小时）</h2>
        <div class="flex h-40 items-end justify-between gap-3">
          <div v-for="(bar, index) in bars" :key="labels[index]" class="flex flex-1 flex-col items-center gap-2">
            <span class="text-[11px] font-bold text-slate-500">{{ bar }}</span>
            <span class="w-full rounded-t-lg bg-gradient-to-t from-blue-600 to-blue-300" :style="{ height: `${bar * 72}px` }" />
            <span class="text-[10px] text-slate-400">{{ labels[index] }}</span>
          </div>
        </div>
      </section>

      <section class="soft-panel mt-5 p-4">
        <h2 class="mb-4 text-sm font-black text-ink">掌握率分布</h2>
        <div class="flex items-center gap-5">
          <div class="relative grid h-28 w-28 place-items-center rounded-full" :style="{ background: `conic-gradient(#3774ff 0 ${masteryRate}%, #63d471 ${masteryRate}% 78%, #ffba3b 78% 100%)` }">
            <div class="grid h-20 w-20 place-items-center rounded-full bg-white text-center">
              <span class="text-2xl font-black text-ink">{{ masteryRate }}%</span>
              <span class="-mt-3 text-[10px] text-slate-400">平均掌握率</span>
            </div>
          </div>
          <div class="flex-1 space-y-3 text-xs">
            <div class="flex items-center justify-between text-slate-500"><span>已掌握（≥80%）</span><b class="text-ink">{{ masteryRate }}%</b></div>
            <div class="flex items-center justify-between text-slate-500"><span>中等（50%-80%）</span><b class="text-ink">32%</b></div>
            <div class="flex items-center justify-between text-slate-500"><span>薄弱（&lt;50%）</span><b class="text-ink">20%</b></div>
          </div>
        </div>
      </section>
    </main>

    <BottomNav />
  </div>
</template>
