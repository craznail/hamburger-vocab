<script setup>
import { useRouter } from 'vue-router'
import { Bell, ChevronRight, Cloud, DatabaseBackup, Eye, HelpCircle, Palette, RotateCcw, Settings as SettingsIcon, SlidersHorizontal } from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
import { downloadDB } from '../api/card'

const router = useRouter()

const groups = [
  {
    title: '学习设置',
    items: [
      { label: '学习设置', icon: SlidersHorizontal },
      { label: '复习设置', icon: RotateCcw },
      { label: '提醒设置', icon: Bell },
      { label: '外观设置', icon: Palette }
    ]
  },
  {
    title: '数据与同步',
    items: [
      { label: '备份与恢复', icon: DatabaseBackup, action: downloadDB },
      { label: '数据导出', icon: Cloud, action: downloadDB }
    ]
  },
  {
    title: '关于应用',
    items: [
      { label: '关于 Recall', icon: Eye, value: 'v0.3.0' },
      { label: '帮助与反馈', icon: HelpCircle }
    ]
  }
]
</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar @back="router.push({ name: 'Profile' })">
      <template #left>
        <h1 class="text-xl font-black text-ink">设置</h1>
      </template>
    </NavBar>

    <main class="flex-1 px-5 pb-8">
      <section v-for="group in groups" :key="group.title" class="mb-5">
        <h2 class="mb-2 px-1 text-xs font-bold text-slate-400">{{ group.title }}</h2>
        <div class="soft-panel overflow-hidden rounded-2xl">
          <button v-for="item in group.items" :key="item.label" class="flex w-full items-center gap-3 border-b border-blue-50 px-4 py-3 text-left last:border-b-0" @click="item.action?.()">
            <span class="grid h-8 w-8 place-items-center rounded-lg bg-blue-50 text-blue-500">
              <component :is="item.icon || SettingsIcon" class="h-4 w-4" />
            </span>
            <span class="flex-1 text-sm font-bold text-ink">{{ item.label }}</span>
            <span v-if="item.value" class="text-xs text-slate-400">{{ item.value }}</span>
            <ChevronRight class="h-4 w-4 text-slate-300" />
          </button>
        </div>
      </section>
    </main>
  </div>
</template>
