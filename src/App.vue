<script setup>
import { onMounted } from 'vue'
import { useAppStore } from './stores/useAppStore'

const store = useAppStore()

onMounted(async () => {
  await store.init()
})
</script>

<template>
  <div class="min-h-screen bg-[#fafafa] text-gray-800">
    <div v-if="!store.initialized" class="flex items-center justify-center h-screen">
      <p class="text-lg text-gray-400">加载中...</p>
    </div>
    <router-view v-else />
  </div>
</template>
