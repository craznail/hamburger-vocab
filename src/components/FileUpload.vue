<script setup>
import { ref } from 'vue'
import { Upload } from 'lucide-vue-next'
import { open } from '@tauri-apps/plugin-dialog'
import { readTxtFile } from '../services/database.js'

const emit = defineEmits(['file-selected'])
const isProcessing = ref(false)

async function handleFileOpen() {
  if (isProcessing.value) return
  isProcessing.value = true

  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Text Files', extensions: ['txt'] }]
    })

    if (!selected) {
      isProcessing.value = false
      return
    }

    const filePath = typeof selected === 'string' ? selected : selected.path

    // Get display name
    let fileName

    // 1) Android: query ContentResolver directly via JS bridge
    if (window.NativeFileResolver) {
      fileName = window.NativeFileResolver.getDisplayName(filePath)
    }

    // 2) Fallback: try stat() (works for content:// URIs)
    if (!fileName) {
      try {
        const { stat } = await import('@tauri-apps/plugin-fs')
        const info = await stat(filePath)
        fileName = info.name
      } catch {}
    }

    // 3) Last resort: extract last path segment
    if (!fileName) {
      fileName = filePath.split(/[/\\]/).pop()
    }

    // Guard: if name is still a URI/ID (has : or URL encoding), use timestamp
    if (!fileName || fileName.includes(':') || /%[0-9A-Fa-f]{2}/.test(fileName)) {
      fileName = String(new Date().getFullYear()) + '.txt'
    }

    const content = await readTxtFile(filePath)
    emit('file-selected', fileName, content)
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
