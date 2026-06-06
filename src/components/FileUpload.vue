<script setup>
import { Upload } from 'lucide-vue-next'

const emit = defineEmits(['file-selected'])

function handleFileChange(event) {
  const file = event.target.files[0]
  if (!file) return
  if (!file.name.endsWith('.txt')) {
    alert('请上传 .txt 文件')
    return
  }
  const reader = new FileReader()
  reader.onload = (e) => {
    emit('file-selected', file.name, e.target.result)
  }
  reader.readAsText(file, 'UTF-8')
  event.target.value = ''
}

function handleDrop(event) {
  event.preventDefault()
  const file = event.dataTransfer.files[0]
  if (!file) return
  if (!file.name.endsWith('.txt')) {
    alert('请上传 .txt 文件')
    return
  }
  const reader = new FileReader()
  reader.onload = (e) => {
    emit('file-selected', file.name, e.target.result)
  }
  reader.readAsText(file, 'UTF-8')
}

function handleDragOver(event) {
  event.preventDefault()
}
</script>

<template>
  <div
    class="border-2 border-dashed border-gray-300 rounded-xl p-8 text-center cursor-pointer hover:border-blue-400 hover:bg-blue-50/50 transition-colors"
    @dragover="handleDragOver"
    @drop="handleDrop"
    @click="$refs.fileInput.click()"
  >
    <Upload class="w-10 h-10 mx-auto mb-3 text-gray-400" />
    <p class="text-gray-500 mb-1">点击选择或拖拽 .txt 文件到此处</p>
    <p class="text-xs text-gray-400">
      支持纯单词 / 单词+释义 / 单词+词形变化+释义
    </p>
    <input
      ref="fileInput"
      type="file"
      accept=".txt"
      class="hidden"
      @change="handleFileChange"
    />
  </div>
</template>
