<script setup>
import { useRoute, useRouter } from 'vue-router'
import { BarChart3, BookOpen, GraduationCap, Home, NotebookPen, UserRound } from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()

const items = [
  { label: '首页', icon: Home, route: 'Home' },
  { label: '知识库', icon: BookOpen, route: 'Library' },
  { label: '学习', icon: GraduationCap, route: 'Study' },
  { label: '错题', icon: NotebookPen, route: 'ErrorNotebook' },
  { label: '统计', icon: BarChart3, route: 'Stats' },
  { label: '我的', icon: UserRound, route: 'Profile' }
]

const routeGroups = {
  Home: ['Home'],
  Library: ['Library', 'DeckDetail', 'Import'],
  Study: ['Study', 'Dictation'],
  ErrorNotebook: ['ErrorNotebook', 'ErrorAdd', 'ErrorDetail', 'ErrorReview'],
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
  <nav class="app-bottom-nav fixed inset-x-0 bottom-0 z-40 pt-2">
    <div class="mx-auto grid max-w-2xl grid-cols-6 gap-1 rounded-[28px] border border-white/70 bg-white/88 px-2 py-2 shadow-[0_-14px_34px_rgba(80,107,173,0.08)] backdrop-blur-2xl">
      <button
        v-for="item in items"
        :key="item.label"
        class="relative flex min-w-0 flex-col items-center gap-1 rounded-2xl px-1 py-2 text-[11px] font-semibold transition-colors"
        :class="isActive(item) ? 'bg-[#f1f5ff] text-blue-600' : 'text-slate-400 hover:text-slate-600'"
        @click="go(item)"
      >
        <span
          class="absolute top-1 h-1 w-6 rounded-full bg-blue-600 transition-opacity"
          :class="isActive(item) ? 'opacity-100' : 'opacity-0'"
        />
        <component
          :is="item.icon"
          class="h-[21px] w-[21px]"
          :stroke-width="isActive(item) ? 2.6 : 2"
        />
        <span class="truncate">{{ item.label }}</span>
      </button>
    </div>
  </nav>
</template>
