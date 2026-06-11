<script setup>
import { ArrowLeft } from 'lucide-vue-next'

defineProps({
  backText: { type: String, default: '返回' },
  showBack: { type: Boolean, default: true }
})

const emit = defineEmits(['back'])
</script>

<template>
  <header class="flex min-h-[72px] items-center justify-between px-5 pt-[calc(env(safe-area-inset-top)+0.75rem)] pb-3">
    <button
      v-if="showBack"
      class="flex h-10 w-10 items-center justify-center rounded-full text-slate-500 transition-colors hover:bg-blue-50 hover:text-blue-600"
      @click="emit('back')"
      :title="backText"
    >
      <ArrowLeft class="h-5 w-5" />
    </button>
    <slot name="left" />
    <slot name="right" />
  </header>
</template>
