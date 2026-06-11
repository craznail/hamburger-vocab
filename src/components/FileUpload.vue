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
    const result = await pickFile()

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
    class="border-2 border-dashed border-gray-300 rounded-xl p-8 text-center hover:border-blue-400 hover:bg-blue-50/50 transition-colors"
    :class="{ 'opacity-50 pointer-events-none': isProcessing }"
    @click="handleFileOpen"
  >
    <Upload class="w-10 h-10 mx-auto mb-3 text-gray-400" />
    <p class="text-gray-500 mb-1">
      {{ isProcessing ? '处理中...' : '点击选择 .txt 文件' }}
    </p>
    <p class="text-xs text-gray-400">
      支持纯单词 / 单词+释义 / 单词+词形变化+释义
    </p>
  </div>
</template>
