<script setup>
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Award, Medal, ShieldCheck, Star, Trophy } from 'lucide-vue-next'
import { useAppStore } from '../stores/useAppStore'
import BottomNav from '../components/BottomNav.vue'
import NavBar from '../components/NavBar.vue'

const router = useRouter()
const store = useAppStore()

onMounted(() => {
  store.refreshAll()
})

const stats = computed(() => store.learningStats || {
  streakDays: 0,
  longestStreak: 0,
  masteredCards: 0,
  totalStudySeconds: 0,
  totalReviews: 0
})
const studyHours = computed(() => stats.value.totalStudySeconds / 3600)
const experience = computed(() => stats.value.totalReviews * 10 + stats.value.masteredCards * 20 + store.decks.length * 50)
const level = computed(() => Math.floor(Math.sqrt(experience.value / 100)) + 1)
const currentLevelStart = computed(() => Math.pow(level.value - 1, 2) * 100)
const nextLevelAt = computed(() => Math.pow(level.value, 2) * 100)
const levelProgress = computed(() => {
  const span = nextLevelAt.value - currentLevelStart.value
  return span ? Math.round(((experience.value - currentLevelStart.value) / span) * 100) : 0
})

const learningAchievements = computed(() => [
  achievement('连续学习 7 天', stats.value.longestStreak, 7, Trophy),
  achievement('连续学习 30 天', stats.value.longestStreak, 30, Medal),
  achievement('掌握 500 卡片', stats.value.masteredCards, 500, Award),
  achievement('掌握 1000 卡片', stats.value.masteredCards, 1000, ShieldCheck),
  achievement('累计学习 10 小时', studyHours.value, 10, Star, 1),
  achievement('累计学习 50 小时', studyHours.value, 50, Trophy, 1)
])

const collectionAchievements = computed(() => [
  achievement('首次导入', store.decks.length, 1, Award),
  achievement('复习 100 次', stats.value.totalReviews, 100, Star),
  achievement('收藏 10 个知识库', store.decks.length, 10, ShieldCheck),
  achievement('单日开始学习', stats.value.totalReviews, 1, Medal)
])

function achievement(label, value, target, icon, decimals = 0) {
  const done = value >= target
  const shown = decimals ? value.toFixed(decimals) : Math.floor(value)
  return { label, done, progress: done ? '已获得' : `${shown}/${target}`, icon }
}
</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar :showBack="false">
      <template #left>
        <h1 class="page-header-title text-[1.85rem]">成就</h1>
      </template>
    </NavBar>

    <main class="flex-1 px-4 pb-8 pt-4">
      <section class="blue-gradient overflow-hidden rounded-[28px] p-4 text-white shadow-[0_22px_40px_rgba(57,96,223,0.18)]">
        <div class="flex items-center gap-4">
          <div class="grid h-16 w-16 place-items-center rounded-full bg-white/20">
            <Trophy class="h-9 w-9 text-yellow-200" />
          </div>
          <div class="min-w-0 flex-1">
            <div class="text-2xl font-black">Lv.{{ level }}</div>
            <p class="mt-1 text-xs text-blue-100">距离 Lv.{{ level + 1 }} 还差 {{ Math.max(0, nextLevelAt - experience) }} 经验值</p>
            <div class="mt-3 h-2 overflow-hidden rounded-full bg-white/20">
              <div class="h-full rounded-full bg-white" :style="{ width: `${levelProgress}%` }" />
            </div>
          </div>
        </div>
      </section>

      <section class="mt-6">
        <div class="mb-3 flex items-center justify-between">
          <h2 class="text-sm font-black text-ink">学习成就</h2>
          <button class="text-xs font-bold text-slate-400">查看全部</button>
        </div>
        <div class="grid grid-cols-3 gap-3">
          <div v-for="item in learningAchievements" :key="item.label" class="tiny-card p-3 text-center">
            <div class="medal mx-auto grid h-11 w-11 place-items-center rounded-full" :class="item.done ? 'bg-gradient-to-br from-yellow-100 to-amber-300 text-amber-600' : 'bg-gradient-to-br from-slate-50 to-blue-100 text-slate-300'">
              <component :is="item.icon" class="h-6 w-6" />
            </div>
            <p class="mt-2 min-h-8 text-[11px] font-bold leading-4 text-ink">{{ item.label }}</p>
            <p class="text-[10px]" :class="item.done ? 'text-amber-500' : 'text-slate-400'">{{ item.progress }}</p>
          </div>
        </div>
      </section>

      <section class="mt-6">
        <div class="mb-3 flex items-center justify-between">
          <h2 class="text-sm font-black text-ink">收集成就</h2>
          <button class="text-xs font-bold text-slate-400">查看全部</button>
        </div>
        <div class="grid grid-cols-4 gap-3">
          <div v-for="item in collectionAchievements" :key="item.label" class="tiny-card p-3 text-center">
            <div class="medal mx-auto grid h-12 w-12 place-items-center rounded-full" :class="item.done ? 'bg-gradient-to-br from-yellow-100 to-amber-300 text-amber-600' : 'bg-gradient-to-br from-blue-50 to-indigo-100 text-slate-300'">
              <component :is="item.icon" class="h-7 w-7" />
            </div>
            <p class="mt-2 truncate text-[10px] font-bold text-slate-500">{{ item.label }}</p>
            <p class="mt-1 text-[10px]" :class="item.done ? 'text-amber-500' : 'text-slate-400'">{{ item.progress }}</p>
          </div>
        </div>
      </section>

      <button class="soft-panel mt-5 flex w-full items-center justify-center p-4 text-sm font-bold text-blue-600" @click="router.push({ name: 'Stats' })">
        查看学习统计
      </button>
    </main>

    <BottomNav />
  </div>
</template>
