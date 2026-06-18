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
        <div>
          <h1 class="page-header-title text-[1.85rem]">学习统计</h1>
          <p class="page-subtitle mt-1">看见节奏，而不是制造压力</p>
        </div>
      </template>
      <template #right>
        <div class="rounded-[18px] border border-blue-100 bg-[#f3f7ff] p-1">
          <button class="rounded-[13px] bg-white px-3 py-1.5 text-xs font-black text-blue-600 shadow-sm">本周</button>
          <button class="px-2 py-1.5 text-xs font-black text-slate-400">本月</button>
        </div>
      </template>
    </NavBar>

    <main class="page-shell">
      <section class="glass-card mb-5 p-5">
        <div class="flex items-start justify-between gap-4">
          <div>
            <p class="section-kicker">本周学习时长</p>
            <div class="mt-3 text-[2.55rem] font-black leading-none text-ink">{{ formatDuration(stats.thisWeekSeconds) }}</div>
            <p class="mt-3 text-sm font-black text-[#2fa88f]">{{ comparison(stats.thisWeekSeconds, stats.previousWeekSeconds) }}</p>
          </div>
          <Clock3 class="h-8 w-8 text-blue-500" />
        </div>
        <div class="mt-5 grid grid-cols-3 gap-2">
          <div class="rounded-[18px] bg-[#f4f7ff] p-3 text-center">
            <div class="text-lg font-black text-ink">{{ stats.thisWeekReviews }}</div>
            <p class="mt-1 text-[10px] font-bold text-slate-400">本周复习</p>
          </div>
          <div class="rounded-[18px] bg-[#eef9f6] p-3 text-center">
            <div class="text-lg font-black text-ink">{{ stats.totalCards }}</div>
            <p class="mt-1 text-[10px] font-bold text-slate-400">总卡片</p>
          </div>
          <div class="rounded-[18px] bg-[#fff7e5] p-3 text-center">
            <div class="text-lg font-black text-ink">{{ stats.accuracyRate }}%</div>
            <p class="mt-1 text-[10px] font-bold text-slate-400">准确率</p>
          </div>
        </div>
      </section>

      <section class="soft-panel mt-5 p-4">
        <h2 class="mb-5 text-sm font-black text-ink">近 7 天学习时长</h2>
        <div class="flex h-28 items-end justify-between gap-3">
          <div v-for="day in stats.dailyActivity" :key="day.date" class="flex flex-1 flex-col items-center gap-2">
            <span class="text-[10px] font-bold text-slate-500">{{ day.studySeconds ? Math.max(1, Math.round(day.studySeconds / 60)) : 0 }}</span>
            <span class="w-full rounded-t-lg bg-gradient-to-t from-blue-600 to-blue-300" :style="{ height: `${Math.max(3, Math.round((day.studySeconds / maxDailySeconds) * 72))}px` }" />
            <span class="text-[10px] text-slate-400">{{ weekday(day.date) }}</span>
          </div>
        </div>
        <p v-if="stats.dailyActivity.length === 0" class="py-6 text-center text-xs text-slate-400">完成一次闪卡学习后，这里会出现趋势</p>
      </section>

      <section class="soft-panel mt-5 p-4">
        <h2 class="mb-4 text-sm font-black text-ink">掌握分布</h2>
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

      <section class="soft-panel mt-5 p-4">
        <div class="flex items-start gap-3">
          <TrendingUp class="mt-1 h-5 w-5 shrink-0 text-blue-500" />
          <div class="min-w-0 flex-1">
            <h2 class="text-sm font-black text-ink">今日建议</h2>
            <p class="mt-1 text-xs leading-5 text-slate-400">先完成 {{ stats.dueCards }} 张复习卡，再进入自由练习。</p>
          </div>
          <button class="rounded-full bg-blue-50 px-3 py-1.5 text-xs font-black text-blue-600" @click="router.push({ name: 'Study' })">去学习</button>
        </div>
      </section>
    </main>

    <BottomNav />
  </div>
</template>
