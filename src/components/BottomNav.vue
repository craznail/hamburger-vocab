<script setup>
import { useRoute, useRouter } from 'vue-router'
import { BarChart3, BookOpen, GraduationCap, Home, UserRound } from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()

const items = [
  { label: '首页', icon: Home, route: 'Home' },
  { label: '知识库', icon: BookOpen, route: 'Library' },
  { label: '学习', icon: GraduationCap, route: 'Study' },
  { label: '统计', icon: BarChart3, route: 'Stats' },
  { label: '我的', icon: UserRound, route: 'Profile' }
]

const routeGroups = {
  Home: ['Home'],
  Library: ['Library', 'DeckDetail', 'Import'],
  Study: ['Study', 'Dictation'],
  Stats: ['Stats', 'Achievements'],
  Profile: ['Profile', 'Settings']
}

function go(item) {
  router.push({ name: item.route })
}

function isActive(item) {
  return routeGroups[item.route]?.includes(route.name)
}
</script>

<template>
  <nav class="fixed inset-x-0 bottom-0 z-40 border-t border-slate-200/80 bg-white/92 px-3 pb-[calc(env(safe-area-inset-bottom)+0.4rem)] pt-1.5 shadow-[0_-8px_24px_rgba(31,43,74,0.06)] backdrop-blur-xl">
    <div class="mx-auto grid max-w-2xl grid-cols-5 gap-1">
      <button
        v-for="item in items"
        :key="item.label"
        class="relative flex min-w-0 flex-col items-center gap-1 rounded-lg px-1 py-1.5 text-[11px] font-semibold transition-colors"
        :class="isActive(item) ? 'text-blue-600' : 'text-slate-400 hover:text-slate-600'"
        @click="go(item)"
      >
        <span
          class="absolute top-0 h-0.5 w-5 rounded-full bg-blue-600 transition-opacity"
          :class="isActive(item) ? 'opacity-100' : 'opacity-0'"
        />
        <component
          :is="item.icon"
          class="h-[22px] w-[22px]"
          :stroke-width="isActive(item) ? 2.6 : 2"
        />
        <span class="truncate">{{ item.label }}</span>
      </button>
    </div>
  </nav>
</template>
