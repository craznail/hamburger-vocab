<script setup>
import { useRouter } from 'vue-router'
import { Award, Medal, ShieldCheck, Star, Trophy } from 'lucide-vue-next'
import BottomNav from '../components/BottomNav.vue'
import NavBar from '../components/NavBar.vue'

const router = useRouter()

const learningAchievements = [
  { label: '连续学习 7 天', done: true, progress: '已获得', icon: Trophy },
  { label: '连续学习 30 天', done: false, progress: '12/30', icon: Medal },
  { label: '掌握 500 卡片', done: false, progress: '235/500', icon: Award },
  { label: '掌握 1000 卡片', done: false, progress: '235/1000', icon: ShieldCheck },
  { label: '累计学习 10 小时', done: false, progress: '6.8/10', icon: Star },
  { label: '累计学习 50 小时', done: false, progress: '6.8/50', icon: Trophy }
]

const collectionAchievements = [
  { label: '导入达人', icon: Award },
  { label: '复习明星', icon: Star },
  { label: '知识收藏家', icon: ShieldCheck },
  { label: '专注勋章', icon: Medal }
]
</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar :showBack="false">
      <template #left>
        <h1 class="text-xl font-black text-ink">成就</h1>
      </template>
    </NavBar>

    <main class="flex-1 px-5 pb-8">
      <section class="blue-gradient overflow-hidden rounded-2xl p-4 text-white">
        <div class="flex items-center gap-4">
          <div class="grid h-16 w-16 place-items-center rounded-full bg-white/20">
            <Trophy class="h-9 w-9 text-yellow-200" />
          </div>
          <div class="min-w-0 flex-1">
            <div class="text-2xl font-black">Lv.8</div>
            <p class="mt-1 text-xs text-blue-100">距离 Lv.9 还差 320 经验值</p>
            <div class="mt-3 h-2 overflow-hidden rounded-full bg-white/20">
              <div class="h-full w-[68%] rounded-full bg-white" />
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
          <div v-for="item in learningAchievements" :key="item.label" class="tiny-card rounded-2xl p-3 text-center">
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
          <div v-for="(item, index) in collectionAchievements" :key="item.label" class="tiny-card rounded-2xl p-3 text-center">
            <div class="medal mx-auto grid h-12 w-12 place-items-center rounded-full" :class="index < 2 ? 'bg-gradient-to-br from-yellow-100 to-amber-300 text-amber-600' : 'bg-gradient-to-br from-blue-50 to-indigo-100 text-blue-400'">
              <component :is="item.icon" class="h-7 w-7" />
            </div>
            <p class="mt-2 truncate text-[10px] font-bold text-slate-500">{{ item.label }}</p>
          </div>
        </div>
      </section>

      <button class="soft-panel mt-5 flex w-full items-center justify-center rounded-2xl p-4 text-sm font-bold text-blue-600" @click="router.push({ name: 'Stats' })">
        查看学习统计
      </button>
    </main>

    <BottomNav />
  </div>
</template>
