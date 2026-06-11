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
  <nav class="fixed inset-x-0 bottom-0 z-40 mx-auto max-w-[430px] border-t border-blue-100/80 bg-white/95 px-5 pb-[calc(env(safe-area-inset-bottom)+0.45rem)] pt-2 shadow-[0_-12px_30px_rgba(82,111,181,0.08)] backdrop-blur">
    <div class="grid grid-cols-5 gap-1">
      <button
        v-for="item in items"
        :key="item.label"
        class="flex min-w-0 flex-col items-center gap-1 rounded-lg px-1 py-1.5 text-[11px] transition-colors"
        :class="isActive(item) ? 'text-blue-600' : 'text-slate-400'"
        @click="go(item)"
      >
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
