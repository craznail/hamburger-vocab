<script setup>
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { BookMarked, Clock3, Flame, TrendingUp } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import BottomNav from '../components/BottomNav.vue'
import NavBar from '../components/NavBar.vue'

const router = useRouter()
const store = useAppStore()

onMounted(() => {
  store.refreshAll()
})

const stats = computed(() => store.learningStats || {
  totalCards: 0,
  masteredCards: 0,
  learningCards: 0,
  newCards: 0,
  dueCards: 0,
  totalReviews: 0,
  totalStudySeconds: 0,
  accuracyRate: 0,
  thisWeekReviews: 0,
  previousWeekReviews: 0,
  thisWeekSeconds: 0,
  previousWeekSeconds: 0,
  dailyActivity: []
})

const masteryRate = computed(() => stats.value.totalCards
  ? Math.round((stats.value.masteredCards / stats.value.totalCards) * 100)
  : 0)
const learningRate = computed(() => stats.value.totalCards
  ? Math.round((stats.value.learningCards / stats.value.totalCards) * 100)
  : 0)
const newRate = computed(() => stats.value.totalCards
  ? Math.max(0, 100 - masteryRate.value - learningRate.value)
  : 0)
const maxDailySeconds = computed(() => Math.max(60, ...stats.value.dailyActivity.map(day => day.studySeconds)))

function formatDuration(seconds) {
  if (!seconds) return '0 分钟'
  if (seconds < 3600) return `${Math.max(1, Math.round(seconds / 60))} 分钟`
  return `${(seconds / 3600).toFixed(1)} 小时`
}

function weekday(date) {
  return new Intl.DateTimeFormat('zh-CN', { weekday: 'short' }).format(new Date(`${date}T00:00:00`))
}

function comparison(current, previous) {
  if (previous === 0) return current > 0 ? '本周开始有记录' : '本周暂无记录'
  const percent = Math.round(((current - previous) / previous) * 100)
  if (percent === 0) return '与上周持平'
  return `较上周 ${percent > 0 ? '↑' : '↓'} ${Math.abs(percent)}%`
}
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
          <Clock3 class="mb-2 h-4 w-4 text-blue-500" />
          <div class="text-2xl font-black text-ink">{{ formatDuration(stats.thisWeekSeconds) }}</div>
          <p class="mt-1 text-xs text-slate-400">学习时长</p>
          <p class="mt-2 text-xs font-bold text-emerald-500">{{ comparison(stats.thisWeekSeconds, stats.previousWeekSeconds) }}</p>
        </div>
        <div class="stat-grid-card p-4">
          <BookMarked class="mb-2 h-4 w-4 text-emerald-500" />
          <div class="text-2xl font-black text-ink">{{ stats.thisWeekReviews }}</div>
          <p class="mt-1 text-xs text-slate-400">本周复习</p>
          <p class="mt-2 text-xs font-bold text-emerald-500">{{ comparison(stats.thisWeekReviews, stats.previousWeekReviews) }}</p>
        </div>
        <div class="stat-grid-card p-4">
          <Flame class="mb-2 h-4 w-4 text-amber-500" />
          <div class="text-2xl font-black text-ink">{{ stats.totalCards }}</div>
          <p class="mt-1 text-xs text-slate-400">总卡片</p>
          <p class="mt-2 text-xs font-bold text-emerald-500">持续积累中</p>
        </div>
        <div class="stat-grid-card p-4">
          <TrendingUp class="mb-2 h-4 w-4 text-red-500" />
          <div class="text-2xl font-black text-ink">{{ stats.accuracyRate }}%</div>
          <p class="mt-1 text-xs text-slate-400">记忆准确率</p>
          <p class="mt-2 text-xs font-bold text-slate-400">累计 {{ stats.totalReviews }} 次评分</p>
        </div>
      </section>

      <section class="soft-panel mt-5 p-4">
        <h2 class="mb-5 text-sm font-black text-ink">近 7 天学习时长</h2>
        <div class="flex h-40 items-end justify-between gap-3">
          <div v-for="day in stats.dailyActivity" :key="day.date" class="flex flex-1 flex-col items-center gap-2">
            <span class="text-[10px] font-bold text-slate-500">{{ day.studySeconds ? Math.max(1, Math.round(day.studySeconds / 60)) : 0 }}</span>
            <span class="w-full rounded-t-lg bg-gradient-to-t from-blue-600 to-blue-300" :style="{ height: `${Math.max(3, Math.round((day.studySeconds / maxDailySeconds) * 96))}px` }" />
            <span class="text-[10px] text-slate-400">{{ weekday(day.date) }}</span>
          </div>
        </div>
        <p v-if="stats.dailyActivity.length === 0" class="py-12 text-center text-xs text-slate-400">完成一次闪卡学习后，这里会出现趋势</p>
      </section>

      <section class="soft-panel mt-5 p-4">
        <h2 class="mb-4 text-sm font-black text-ink">掌握率分布</h2>
        <div class="flex items-center gap-5">
          <div class="relative grid h-28 w-28 place-items-center rounded-full" :style="{ background: `conic-gradient(#3774ff 0 ${masteryRate}%, #63d471 ${masteryRate}% ${masteryRate + learningRate}%, #ffba3b ${masteryRate + learningRate}% 100%)` }">
            <div class="grid h-20 w-20 place-items-center rounded-full bg-white text-center">
              <span class="text-2xl font-black text-ink">{{ masteryRate }}%</span>
              <span class="-mt-3 text-[10px] text-slate-400">平均掌握率</span>
            </div>
          </div>
          <div class="flex-1 space-y-3 text-xs">
            <div class="flex items-center justify-between text-slate-500"><span>已掌握</span><b class="text-ink">{{ stats.masteredCards }}（{{ masteryRate }}%）</b></div>
            <div class="flex items-center justify-between text-slate-500"><span>学习中</span><b class="text-ink">{{ stats.learningCards }}（{{ learningRate }}%）</b></div>
            <div class="flex items-center justify-between text-slate-500"><span>新卡片</span><b class="text-ink">{{ stats.newCards }}（{{ newRate }}%）</b></div>
          </div>
        </div>
      </section>
    </main>

    <BottomNav />
  </div>
</template>
