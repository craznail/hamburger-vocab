<script setup lang="ts">
import { computed } from 'vue'
import { renderRichText } from '../utils/richText'

const props = withDefaults(defineProps<{
  text?: string | null
  fallback?: string
}>(), {
  text: '',
  fallback: '',
})

const html = computed(() => renderRichText(props.text, props.fallback))
</script>

<template>
  <div class="rich-text" v-html="html"></div>
</template>

<style scoped>
.rich-text {
  min-width: 0;
  overflow-wrap: anywhere;
}

.rich-text :deep(p) {
  margin: 0;
  white-space: pre-wrap;
}

.rich-text :deep(p + p) {
  margin-top: 0.65em;
}

.rich-text :deep(.katex) {
  font-size: 1em;
}

.rich-text :deep(.katex-display) {
  margin: 0.75em 0;
  overflow-x: auto;
  overflow-y: hidden;
}
</style>
