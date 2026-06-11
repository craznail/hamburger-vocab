<script setup>
import { ref } from 'vue'
import { Upload } from 'lucide-vue-next'
import { pickFile, readTxtFile } from '../platform/file'

const emit = defineEmits(['file-selected'])
const isProcessing = ref(false)

async function handleFileOpen() {
  if (isProcessing.value) return
  isProcessing.value = true

  try {
    const result = await pickFile({
      filters: [{ name: 'Text Files', extensions: ['txt', 'md', 'markdown'] }]
    })

    if (!result) {
      isProcessing.value = false
      return
    }

    const content = await readTxtFile(result.path)
    emit('file-selected', result.name, content)
  } catch (e) {
    console.error('File open error:', e)
    alert('无法读取文件：' + e)
  } finally {
    isProcessing.value = false
  }
}
</script>

<template>
  <div
    class="rounded-xl p-4 text-center transition-colors"
    :class="{ 'opacity-50 pointer-events-none': isProcessing }"
    @click="handleFileOpen"
  >
    <Upload class="mx-auto mb-3 h-9 w-9 text-blue-400" />
    <p class="mb-1 text-sm font-black text-ink">
      {{ isProcessing ? '处理中...' : '拖拽文件到这里' }}
    </p>
    <p class="text-xs text-slate-400">
      支持 TXT、Markdown
    </p>
    <button class="blue-gradient mt-5 h-10 rounded-xl px-8 text-sm font-bold text-white" type="button">
      选择文件
    </button>
  </div>
</template>
