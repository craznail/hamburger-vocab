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
  Study: ['Study', 'WordReview', 'Dictation'],
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
  <nav class="app-bottom-nav fixed inset-x-0 bottom-0 z-40">
    <div class="mobile-tabbar grid grid-cols-6 gap-1">
      <button
        v-for="item in items"
        :key="item.label"
        class="mobile-tab-item"
        :class="{ 'mobile-tab-item-active': isActive(item) }"
        @click="go(item)"
      >
        <span
          class="mobile-tab-indicator"
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
