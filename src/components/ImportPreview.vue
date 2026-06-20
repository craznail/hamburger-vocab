<script setup>
import { computed } from 'vue'

const props = defineProps({
  result: { type: Object, required: true },
  fileName: { type: String, required: true }
})

const emit = defineEmits(['confirm', 'cancel'])

const formatLabel = computed(() => {
  const labels = {
    D: '分块格式（空行分隔）',
    unknown: '未识别'
  }
  return labels[props.result.format] || '未识别'
})
</script>

<template>
  <div class="fixed inset-0 bg-black/40 flex items-center justify-center z-50 p-4" @click.self="emit('cancel')">
    <div class="bg-white rounded-xl shadow-xl max-w-md w-full p-6">
      <h3 class="text-lg font-semibold mb-2">导入预览</h3>
      <p class="text-sm text-gray-500 mb-4">
        文件：<span class="font-medium">{{ fileName }}</span>
      </p>

      <div class="space-y-2 text-sm mb-4">
        <div class="flex justify-between">
          <span class="text-gray-500">识别格式</span>
          <span class="font-medium">{{ formatLabel }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-500">单词总数</span>
          <span class="font-medium">{{ result.validCount }}</span>
        </div>
        <div v-if="result.errors.length > 0" class="flex justify-between">
          <span class="text-red-500">解析问题</span>
          <span class="text-red-500 font-medium">{{ result.errors.length }}</span>
        </div>
      </div>

      <!-- Preview rows -->
      <div class="bg-gray-50 rounded-lg p-3 mb-4">
        <p class="text-xs text-gray-400 mb-2">预览（前 {{ result.preview.length }} 条）</p>
        <div v-for="(row, i) in result.preview" :key="i" class="text-sm py-1">
          <span class="font-medium">{{ row.word }}</span>
          <span v-if="row.inflections.length" class="text-gray-400 ml-1">
            · {{ row.inflections.join(' · ') }}
          </span>
          <span v-if="row.definition" class="text-gray-500 ml-2">
            {{ row.definition }}
          </span>
        </div>
      </div>

      <!-- Errors -->
      <div v-if="result.errors.length > 0" class="bg-red-50 rounded-lg p-3 mb-4 text-sm">
        <p class="text-red-600 font-medium mb-1">{{ result.errors.length }} 个解析问题：</p>
        <div v-for="(err, i) in result.errors.slice(0, 3)" :key="i" class="text-red-500 text-xs py-0.5">
          第 {{ err.line }} 行：{{ err.msg }}
        </div>
      </div>

      <div class="flex gap-3">
        <button
          class="flex-1 px-4 py-2 rounded-lg border border-gray-300 text-gray-600 hover:bg-gray-50 cursor-pointer"
          @click="emit('cancel')"
        >
          取消
        </button>
        <button
          class="flex-1 px-4 py-2 rounded-lg bg-blue-600 text-white hover:bg-blue-700 cursor-pointer"
          @click="emit('confirm')"
        >
          导入 {{ result.validCount }} 个单词
        </button>
      </div>
    </div>
  </div>
</template>
