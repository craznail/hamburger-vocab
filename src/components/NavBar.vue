<script setup>
import { ArrowLeft } from 'lucide-vue-next'

defineProps({
  backText: { type: String, default: '返回' },
  showBack: { type: Boolean, default: true }
})

const emit = defineEmits(['back'])
</script>

<template>
  <header class="sticky top-0 z-30 flex min-h-[72px] items-center justify-between border-b border-slate-200/70 bg-[#f8faff]/90 px-5 pt-[calc(env(safe-area-inset-top)+0.65rem)] pb-3 backdrop-blur-xl">
    <button
      v-if="showBack"
      class="flex h-10 w-10 items-center justify-center rounded-lg border border-slate-200 bg-white text-slate-500 shadow-sm transition-colors hover:border-blue-200 hover:text-blue-600"
      @click="emit('back')"
      :title="backText"
    >
      <ArrowLeft class="h-5 w-5" />
    </button>
    <slot name="left" />
    <slot name="right" />
  </header>
</template>
