<script setup>
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  ArrowRight,
  CalendarDays,
  Clock3,
} from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import BottomNav from '../components/BottomNav.vue'
import { getAppSettings } from '../platform/appSettings'

const router = useRouter()
const store = useAppStore()
const statsHeroArt = new URL('../assets/hero/stats-hero-bg.png', import.meta.url).href
const suggestionIcon = new URL('../assets/icons/stats-suggestion-icon.svg', import.meta.url).href
const streakIcon = new URL('../assets/icons/stats-summary-streak.svg', import.meta.url).href
const recordIcon = new URL('../assets/icons/stats-summary-record.svg', import.meta.url).href
const correctIcon = new URL('../assets/icons/stats-summary-correct.svg', import.meta.url).href
const appSettings = getAppSettings()
const useMockData = computed(() => appSettings.stats?.enableMockDataFallback === true)

const rangeTabs = ['本周', '本月', '全部']

const emptyStats = {
  totalCards: 0,
  masteredCards: 0,
  learningCards: 0,
  newCards: 0,
  dueCards: 0,
  totalReviews: 0,
  totalStudySeconds: 0,
  accuracyRate: 0,
  streakDays: 0,
  longestStreak: 0,
  thisWeekReviews: 0,
  previousWeekReviews: 0,
  thisWeekSeconds: 0,
  previousWeekSeconds: 0,
  dailyActivity: [],
}

const mockStats = {
  totalCards: 188,
  masteredCards: 128,
  learningCards: 42,
  newCards: 0,
  dueCards: 18,
  totalReviews: 512,
  totalStudySeconds: 48600,
  accuracyRate: 91,
  streakDays: 7,
  longestStreak: 14,
  thisWeekReviews: 126,
  previousWeekReviews: 107,
  thisWeekSeconds: 13320,
  previousWeekSeconds: 11280,
  dailyActivity: [
    { date: '2026-06-15', reviewCount: 18, studySeconds: 1500 },
    { date: '2026-06-16', reviewCount: 34, studySeconds: 3900 },
    { date: '2026-06-17', reviewCount: 22, studySeconds: 2100 },
    { date: '2026-06-18', reviewCount: 36, studySeconds: 4200 },
    { date: '2026-06-19', reviewCount: 28, studySeconds: 3300 },
    { date: '2026-06-20', reviewCount: 41, studySeconds: 4800 },
    { date: '2026-06-21', reviewCount: 12, studySeconds: 720 },
  ],
}

onMounted(() => {
  if (useMockData.value) return
  store.refreshAll()
})

const stats = computed(() => useMockData.value ? mockStats : (store.learningStats || emptyStats))

const masteryRate = computed(() => stats.value.totalCards
  ? Math.round((stats.value.masteredCards / stats.value.totalCards) * 100)
  : 0)

const learningRate = computed(() => stats.value.totalCards
  ? Math.round((stats.value.learningCards / stats.value.totalCards) * 100)
  : 0)

const dueRate = computed(() => stats.value.totalCards
  ? Math.round((stats.value.dueCards / stats.value.totalCards) * 100)
  : 0)

const maxDailySeconds = computed(() => Math.max(60, ...stats.value.dailyActivity.map(day => Number(day.studySeconds) || 0)))
const maxDailyIndex = computed(() => {
  let bestIndex = 0
  let bestValue = -1
  stats.value.dailyActivity.forEach((day, index) => {
    const current = Number(day.studySeconds) || 0
    if (current > bestValue) {
      bestValue = current
      bestIndex = index
    }
  })
  return bestIndex
})

const durationParts = computed(() => {
  const totalMinutes = Math.max(0, Math.round((Number(stats.value.thisWeekSeconds) || 0) / 60))
  return {
    hours: Math.floor(totalMinutes / 60),
    minutes: totalMinutes % 60,
  }
})

const weeklyChange = computed(() => {
  const current = Number(stats.value.thisWeekSeconds) || 0
  const previous = Number(stats.value.previousWeekSeconds) || 0
  if (previous <= 0) {
    return { positive: true, text: current > 0 ? '↑ 100%' : '0%' }
  }
  const percent = Math.round(((current - previous) / previous) * 100)
  return {
    positive: percent >= 0,
    text: `${percent >= 0 ? '↑' : '↓'} ${Math.abs(percent)}%`,
  }
})

const chartDays = computed(() => stats.value.dailyActivity.map((day, index) => ({
  ...day,
  label: shortWeekday(day.date),
  minutesLabel: formatChartDuration(day.studySeconds),
  height: Math.max(14, Math.round(((Number(day.studySeconds) || 0) / maxDailySeconds.value) * 118)),
  isPeak: index === maxDailyIndex.value,
})))

const distributionItems = computed(() => ([
  {
    label: '已掌握',
    count: Number(stats.value.masteredCards) || 0,
    percent: masteryRate.value,
    color: '#2f6cff',
    softColor: 'rgba(47, 108, 255, 0.24)',
  },
  {
    label: '学习中',
    count: Number(stats.value.learningCards) || 0,
    percent: learningRate.value,
    color: '#44d1a0',
    softColor: 'rgba(68, 209, 160, 0.24)',
  },
  {
    label: '待复习',
    count: Number(stats.value.dueCards) || 0,
    percent: dueRate.value,
    color: '#ffa63d',
    softColor: 'rgba(255, 166, 61, 0.24)',
  },
]))

const ringGradient = computed(() => {
  const blueEnd = Math.max(0, masteryRate.value)
  const greenStart = Math.min(100, blueEnd + 2)
  const greenEnd = Math.min(100, greenStart + learningRate.value)
  const orangeStart = Math.min(100, greenEnd + 2)

  return `conic-gradient(
    #2f6cff 0 ${blueEnd}%,
    #ffffff ${blueEnd}% ${greenStart}%,
    #44d1a0 ${greenStart}% ${greenEnd}%,
    #ffffff ${greenEnd}% ${orangeStart}%,
    #ffa63d ${orangeStart}% 100%
  )`
})

const weeklyCorrectionCount = computed(() => useMockData.value ? 36 : (Number(stats.value.dueCards) || 0))

function shortWeekday(date) {
  return ['日', '一', '二', '三', '四', '五', '六'][new Date(`${date}T00:00:00`).getDay()]
}

function formatChartDuration(seconds) {
  const mins = Math.max(0, Math.round((Number(seconds) || 0) / 60))
  if (mins >= 60) {
    const hours = Math.floor(mins / 60)
    const rest = mins % 60
    return `${hours}h ${String(rest).padStart(2, '0')}m`
  }
  return `${mins}m`
}
</script>

<template>
  <div class="stats-page app-page flex min-h-screen flex-col">
    <div class="stats-shell">
      <header class="stats-topbar">
        <div class="min-w-0">
          <h1 class="stats-title">学习统计</h1>
          <p class="stats-subtitle">看见节奏，而不是制造压力</p>
        </div>
        <button class="stats-calendar-button" type="button" aria-label="统计日历">
          <CalendarDays class="h-5 w-5" />
        </button>
      </header>

      <div class="stats-range-tabs" role="tablist" aria-label="统计范围">
        <button
          v-for="tab in rangeTabs"
          :key="tab"
          class="stats-range-tab"
          :class="{ 'stats-range-tab-active': tab === '本周' }"
          type="button"
        >
          {{ tab }}
        </button>
      </div>

      <section class="stats-focus-card" :style="{ '--hero-image': `url(${statsHeroArt})` }">
        <div class="stats-focus-top">
          <div class="stats-focus-copy">
            <p class="stats-focus-kicker">
              <Clock3 class="h-5 w-5" />
              本周学习时长
            </p>

            <div class="stats-focus-time">
              <span class="stats-focus-time-number">{{ durationParts.hours }}</span>
              <span class="stats-focus-time-unit">h</span>
              <span class="stats-focus-time-number stats-focus-time-number-minutes">{{ String(durationParts.minutes).padStart(2, '0') }}</span>
              <span class="stats-focus-time-unit">m</span>
            </div>

            <p class="stats-focus-change">
              比上周
              <span :class="{ 'stats-focus-change-positive': weeklyChange.positive, 'stats-focus-change-negative': !weeklyChange.positive }">
                {{ weeklyChange.text }}
              </span>
            </p>
          </div>
        </div>

        <div class="stats-focus-chart">
          <h2 class="stats-section-title">近 7 天学习时长</h2>

          <div class="stats-chart-shell">
            <div class="stats-chart-gridline stats-chart-gridline-top">
              <span>2h</span>
            </div>
            <div class="stats-chart-gridline stats-chart-gridline-mid">
              <span>1h</span>
            </div>
            <div class="stats-chart-gridline stats-chart-gridline-base">
              <span>0</span>
            </div>

            <div class="stats-chart-bars">
              <div v-for="day in chartDays" :key="day.date" class="stats-chart-day">
                <span class="stats-chart-value">{{ day.minutesLabel }}</span>
                <span class="stats-chart-bar" :class="{ 'stats-chart-bar-peak': day.isPeak }" :style="{ height: `${day.height}px` }" />
                <span class="stats-chart-label">{{ day.label }}</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section class="stats-distribution-card">
        <h2 class="stats-section-title stats-distribution-title">掌握分布</h2>

        <div class="stats-distribution-body">
          <div class="stats-ring-wrap">
            <div class="stats-ring" :style="{ background: ringGradient }">
              <div class="stats-ring-inner">
                <span class="stats-ring-total">{{ stats.totalCards }}</span>
                <span class="stats-ring-caption">总计</span>
              </div>
            </div>
          </div>

          <div class="stats-distribution-list">
            <div v-for="item in distributionItems" :key="item.label" class="stats-distribution-item">
              <div class="stats-distribution-label">
                <span class="stats-distribution-dot" :style="{ background: item.color }" />
                <span>{{ item.label }}</span>
              </div>
              <div class="stats-distribution-track">
                <span class="stats-distribution-fill" :style="{ width: `${Math.max(18, item.percent)}%`, background: `linear-gradient(90deg, ${item.softColor} 0%, ${item.color} 100%)` }" />
              </div>
              <div class="stats-distribution-value">{{ item.count }}</div>
              <div class="stats-distribution-percent">{{ item.percent }}%</div>
            </div>
          </div>
        </div>
      </section>

      <section class="stats-suggestion-card">
        <img class="stats-suggestion-icon" :src="suggestionIcon" alt="" aria-hidden="true" />
        <div class="stats-suggestion-copy">
          <h2 class="stats-suggestion-title">今日建议</h2>
          <p class="stats-suggestion-text">先完成 {{ stats.dueCards }} 张复习卡，再进入自由练习。</p>
        </div>
        <button class="stats-suggestion-action" type="button" @click="router.push({ name: 'Study' })">
          去学习
          <ArrowRight class="h-4 w-4" />
        </button>
      </section>

      <section class="stats-summary-card">
        <div class="stats-summary-item">
          <img class="stats-summary-icon" :src="streakIcon" alt="" aria-hidden="true" />
          <div class="stats-summary-copy">
            <p class="stats-summary-value">连续 <strong>{{ stats.streakDays }}</strong> 天</p>
            <p class="stats-summary-label">坚持学习</p>
          </div>
        </div>

        <div class="stats-summary-item">
          <img class="stats-summary-icon" :src="recordIcon" alt="" aria-hidden="true" />
          <div class="stats-summary-copy">
            <p class="stats-summary-value">累计 <strong>{{ stats.totalReviews }}</strong> 次</p>
            <p class="stats-summary-label">学习记录</p>
          </div>
        </div>

        <div class="stats-summary-item">
          <img class="stats-summary-icon" :src="correctIcon" alt="" aria-hidden="true" />
          <div class="stats-summary-copy">
            <p class="stats-summary-value">错题订正 <strong>{{ weeklyCorrectionCount }}</strong></p>
            <p class="stats-summary-label">本周订正</p>
          </div>
        </div>
      </section>
    </div>

    <BottomNav />
  </div>
</template>

<style scoped>
.stats-page {
  background:
    linear-gradient(rgba(89, 122, 194, 0.01) 1px, transparent 1px),
    linear-gradient(90deg, rgba(89, 122, 194, 0.01) 1px, transparent 1px),
    radial-gradient(circle at 22% 0%, rgba(110, 154, 245, 0.07), transparent 32%),
    linear-gradient(180deg, #fffefe 0%, #f7faff 42%, #eef4fc 100%);
  background-size: 28px 28px, 28px 28px, auto, auto;
}

.stats-shell {
  flex: 1;
  padding: calc(var(--safe-area-top) + 0.92rem) 1rem 5.15rem;
}

.stats-topbar {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.stats-title {
  margin: 0;
  color: #152b62;
  font-size: 1.84rem;
  line-height: 1.02;
  font-weight: 950;
}

.stats-subtitle {
  margin: 0.5rem 0 0;
  color: #6178a9;
  font-size: 0.96rem;
  line-height: 1.4;
  font-weight: 650;
}

.stats-calendar-button {
  display: grid;
  width: 3.36rem;
  height: 3.36rem;
  flex: 0 0 auto;
  place-items: center;
  border: 1px solid rgba(214, 223, 241, 0.95);
  border-radius: 1.12rem;
  background: rgba(255, 255, 255, 0.92);
  color: #326cff;
  box-shadow:
    0 10px 24px rgba(82, 106, 159, 0.05),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.stats-range-tabs {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 0.22rem;
  margin-top: 1.08rem;
  padding: 0.28rem;
  border: 1px solid rgba(215, 225, 244, 0.94);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.9);
  box-shadow:
    0 12px 24px rgba(84, 110, 168, 0.04),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.stats-range-tab {
  min-height: 3.14rem;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: #1c3269;
  font-size: 1rem;
  font-weight: 850;
}

.stats-range-tab-active {
  background: linear-gradient(135deg, #3774ff 0%, #245dff 100%);
  color: white;
  box-shadow:
    0 12px 20px rgba(40, 94, 234, 0.24),
    inset 0 1px 0 rgba(255, 255, 255, 0.24);
}

.stats-focus-card,
.stats-distribution-card,
.stats-suggestion-card,
.stats-summary-card {
  position: relative;
  overflow: hidden;
  border: 1px solid rgba(214, 227, 255, 0.95);
  border-radius: 1.7rem;
  background:
    linear-gradient(rgba(74, 113, 170, 0.02) 1px, transparent 1px),
    linear-gradient(90deg, rgba(74, 113, 170, 0.02) 1px, transparent 1px),
    linear-gradient(180deg, rgba(255, 255, 255, 0.96) 0%, rgba(246, 249, 255, 0.96) 100%);
  background-size: 20px 20px, 20px 20px, auto;
  box-shadow:
    0 18px 36px rgba(73, 100, 158, 0.07),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.stats-focus-card {
  margin-top: 1.18rem;
  padding: 1.08rem 1.08rem 1rem;
}

.stats-focus-card::before {
  content: "";
  position: absolute;
  inset: 0 0 38% 0;
  background-image:
    linear-gradient(180deg, rgba(255, 255, 255, 0.42) 0%, rgba(248, 251, 255, 0.78) 74%, rgba(248, 251, 255, 0.92) 100%),
    var(--hero-image);
  background-position: right -0.34rem top 1.72rem;
  background-repeat: no-repeat;
  background-size: 96% auto;
  filter: saturate(0.98) brightness(1.02);
  pointer-events: none;
}

.stats-focus-card > * {
  position: relative;
  z-index: 1;
}

.stats-focus-top {
  min-height: 14.4rem;
}

.stats-focus-copy {
  max-width: 12rem;
}

.stats-focus-kicker {
  display: inline-flex;
  align-items: center;
  gap: 0.56rem;
  margin: 0;
  color: #1a3270;
  font-size: 0.92rem;
  font-weight: 900;
}

.stats-focus-time {
  display: flex;
  align-items: flex-end;
  gap: 0.18rem;
  margin-top: 1.08rem;
  white-space: nowrap;
}

.stats-focus-time-number {
  color: #2762ff;
  font-size: 5.22rem;
  line-height: 0.9;
  font-weight: 950;
  letter-spacing: -0.08em;
}

.stats-focus-time-number-minutes {
  margin-left: 0.1rem;
}

.stats-focus-time-unit {
  margin: 0 0.18rem 0.44rem 0;
  color: #2762ff;
  font-size: 0.98rem;
  line-height: 1;
  font-weight: 900;
}

.stats-focus-change {
  display: flex;
  align-items: center;
  gap: 0.82rem;
  margin: 0.86rem 0 0;
  color: #48608f;
  font-size: 0.92rem;
  font-weight: 800;
}

.stats-focus-change-positive {
  color: #1fc882;
}

.stats-focus-change-negative {
  color: #f06767;
}

.stats-focus-chart {
  margin-top: 0.34rem;
  padding-top: 1.14rem;
  border-top: 1px solid rgba(221, 231, 247, 0.96);
}

.stats-section-title {
  margin: 0;
  color: #152b62;
  font-size: 0.98rem;
  font-weight: 900;
}

.stats-chart-shell {
  position: relative;
  height: 14.78rem;
  margin-top: 1rem;
}

.stats-chart-gridline {
  position: absolute;
  left: 2.05rem;
  right: 0;
  border-top: 1px dashed rgba(215, 225, 244, 0.86);
}

.stats-chart-gridline span {
  position: absolute;
  left: -1.78rem;
  top: -0.78rem;
  color: #5e74a3;
  font-size: 0.8rem;
  font-weight: 700;
}

.stats-chart-gridline-top {
  top: 0.82rem;
}

.stats-chart-gridline-mid {
  top: 6.98rem;
}

.stats-chart-gridline-base {
  top: 13.08rem;
  border-top-style: solid;
}

.stats-chart-bars {
  position: absolute;
  left: 2.05rem;
  right: 0.1rem;
  bottom: 0;
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  align-items: end;
  gap: 0.72rem;
  height: 100%;
}

.stats-chart-day {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-end;
  gap: 0.68rem;
  height: 100%;
}

.stats-chart-value {
  color: #35569e;
  font-size: 0.8rem;
  font-weight: 800;
}

.stats-chart-bar {
  width: 100%;
  max-width: 1.76rem;
  border-radius: 0.54rem 0.54rem 0 0;
  background: linear-gradient(180deg, #6f9eff 0%, #245dff 100%);
  box-shadow:
    0 8px 18px rgba(49, 98, 233, 0.18),
    inset 0 1px 0 rgba(255, 255, 255, 0.26);
}

.stats-chart-bar-peak {
  background: linear-gradient(180deg, #5fe0bc 0%, #2dcf9a 100%);
  box-shadow:
    0 8px 18px rgba(45, 207, 154, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.stats-chart-label {
  color: #596f9f;
  font-size: 0.78rem;
  font-weight: 700;
}

.stats-distribution-card {
  margin-top: 1rem;
  padding: 1.08rem 1rem 1.2rem;
}

.stats-distribution-title {
  margin-bottom: 1rem;
}

.stats-distribution-body {
  display: flex;
  align-items: center;
  gap: 1.08rem;
  min-height: 11rem;
}

.stats-ring-wrap {
  flex: 0 0 auto;
}

.stats-ring {
  display: grid;
  width: 10.95rem;
  height: 10.95rem;
  place-items: center;
  border-radius: 999px;
}

.stats-ring-inner {
  display: grid;
  width: 7.45rem;
  height: 7.45rem;
  place-items: center;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.96);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.96);
}

.stats-ring-total {
  color: #172e65;
  font-size: 2.4rem;
  line-height: 1;
  font-weight: 950;
}

.stats-ring-caption {
  margin-top: -0.8rem;
  color: #7b8db3;
  font-size: 0.92rem;
  font-weight: 700;
}

.stats-distribution-list {
  display: grid;
  flex: 1;
  gap: 1.04rem;
  min-width: 0;
}

.stats-distribution-item {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto auto;
  align-items: center;
  gap: 0.9rem;
}

.stats-distribution-label {
  display: inline-flex;
  align-items: center;
  gap: 0.52rem;
  min-width: 4.5rem;
  color: #273f7c;
  font-size: 0.92rem;
  font-weight: 800;
}

.stats-distribution-dot {
  width: 0.82rem;
  height: 0.82rem;
  border-radius: 999px;
  flex: 0 0 auto;
}

.stats-distribution-track {
  height: 0.42rem;
  overflow: hidden;
  border-radius: 999px;
  background: rgba(232, 238, 248, 0.9);
}

.stats-distribution-fill {
  display: block;
  height: 100%;
  border-radius: inherit;
}

.stats-distribution-value {
  color: #172e65;
  font-size: 0.94rem;
  font-weight: 900;
}

.stats-distribution-percent {
  color: #7084b0;
  font-size: 0.88rem;
  font-weight: 700;
}

.stats-suggestion-card {
  display: flex;
  align-items: center;
  gap: 0.84rem;
  margin-top: 1rem;
  padding: 1.08rem 0.98rem;
  min-height: 6.2rem;
}

.stats-suggestion-icon {
  width: 3.72rem;
  height: 3.72rem;
  flex: 0 0 auto;
}

.stats-suggestion-copy {
  min-width: 0;
  flex: 1;
}

.stats-suggestion-title {
  margin: 0;
  color: #182f67;
  font-size: 1.02rem;
  font-weight: 900;
}

.stats-suggestion-text {
  margin: 0.4rem 0 0;
  color: #687da9;
  font-size: 0.84rem;
  line-height: 1.36;
  font-weight: 650;
}

.stats-suggestion-action {
  display: inline-flex;
  align-items: center;
  gap: 0.36rem;
  min-height: 2.84rem;
  padding: 0 1rem;
  border: 0;
  border-radius: 999px;
  background: linear-gradient(135deg, #3774ff 0%, #245dff 100%);
  color: white;
  font-size: 0.9rem;
  font-weight: 900;
  box-shadow: 0 14px 24px rgba(44, 97, 234, 0.22);
}

.stats-summary-card {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  margin-top: 1rem;
  padding: 1.08rem 0.12rem;
  min-height: 6.3rem;
}

.stats-summary-item {
  display: flex;
  align-items: center;
  gap: 0.46rem;
  min-width: 0;
  padding: 0 0.52rem;
}

.stats-summary-item + .stats-summary-item {
  border-left: 1px solid rgba(223, 231, 247, 0.94);
}

.stats-summary-icon {
  width: 2.64rem;
  height: 2.64rem;
  flex: 0 0 auto;
}

.stats-summary-copy {
  min-width: 0;
}

.stats-summary-value {
  margin: 0;
  color: #1a2f63;
  font-size: 0.72rem;
  font-weight: 800;
  line-height: 1.1;
  white-space: nowrap;
}

.stats-summary-value strong {
  font-size: 1rem;
  font-weight: 950;
  color: #142a61;
}

.stats-summary-label {
  margin: 0.22rem 0 0;
  color: #788aaf;
  font-size: 0.68rem;
  font-weight: 650;
  white-space: nowrap;
}
</style>
