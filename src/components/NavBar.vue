<script setup>
import { ArrowLeft } from 'lucide-vue-next'

defineProps({
  backText: { type: String, default: '返回' },
  showBack: { type: Boolean, default: true }
})

const emit = defineEmits(['back'])
</script>

<template>
  <header class="flex items-center justify-between px-5 pt-8 pb-4 border-b border-gray-100">
    <button
      v-if="showBack"
      class="flex items-center gap-2 py-1 text-gray-600 hover:text-gray-900 transition-colors cursor-pointer"
      @click="emit('back')"
    >
      <ArrowLeft class="w-6 h-6" />
      <span class="text-base font-medium">{{ backText }}</span>
    </button>
    <slot name="left" />
    <slot name="right" />
  </header>
</template>
