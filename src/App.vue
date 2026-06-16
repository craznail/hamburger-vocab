<script setup>
import { onMounted } from 'vue'
import { useAppStore } from './stores/useAppStore'
import { useErrorNotebookStore } from './stores/useErrorNotebookStore'

const store = useAppStore()
const errorNotebookStore = useErrorNotebookStore()

onMounted(async () => {
  await store.init()
  errorNotebookStore.prime()
})
</script>

<template>
  <div class="min-h-screen">
    <div v-if="!store.initialized" class="app-shell flex items-center justify-center">
      <p class="text-lg text-slate-400">加载中...</p>
    </div>
    <main v-else class="app-shell">
      <router-view />
    </main>
  </div>
</template>
