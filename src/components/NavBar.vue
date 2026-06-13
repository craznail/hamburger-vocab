<script setup>
import { ArrowLeft } from 'lucide-vue-next'

defineProps({
  backText: { type: String, default: '返回' },
  showBack: { type: Boolean, default: true }
})

const emit = defineEmits(['back'])
</script>

<template>
  <header class="app-navbar sticky top-0 z-30 px-4">
    <div class="mx-auto flex min-h-[64px] items-center justify-between rounded-[26px] border border-white/70 bg-white/74 px-4 py-2 shadow-[0_12px_30px_rgba(94,123,189,0.08)] backdrop-blur-2xl">
      <div class="flex min-w-0 items-center gap-3">
        <button
          v-if="showBack"
          class="grid h-10 w-10 shrink-0 place-items-center rounded-full border border-[#d9e5ff] bg-white text-[#5a71ab] shadow-[0_8px_18px_rgba(95,126,194,0.08)] transition-colors hover:border-blue-200 hover:text-blue-600"
          @click="emit('back')"
          :title="backText"
        >
          <ArrowLeft class="h-4.5 w-4.5" />
        </button>
        <slot name="left" />
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <slot name="right" />
      </div>
    </div>
  </header>
</template>
