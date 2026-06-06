<script setup>
defineProps({
  current: { type: Number, default: 0 },
  total: { type: Number, default: 0 }
})
</script>

<template>
  <div class="w-full bg-gray-200 rounded-full h-2" v-if="total > 0">
    <div
      class="bg-blue-500 h-2 rounded-full transition-all duration-300"
      :style="{ width: `${Math.round((current / total) * 100)}%` }"
    />
  </div>
</template>
