<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'
import {
  Brain,
  Clock3,
  Lightbulb,
  NotebookPen,
  Save,
  Sparkles,
  Tag,
  Target,
} from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
import * as errorApi from '../api/errorItem'

const route = useRoute()
const router = useRouter()
const item = ref<errorApi.ErrorItem | null>(null)
const saving = ref(false)
const saveMessage = ref('')
const form = ref({
  questionText: '',
  answerText: '',
  analysis: '',
  mistakeAnalysis: '',
  userNotes: '',
  knowledgePointsText: '',
})

const imageSrc = computed(() => item.value?.localImagePath ? convertFileSrc(item.value.localImagePath) : item.value?.remoteImageUrl || '')
const knowledgePoints = computed(() => form.value.knowledgePointsText.split(/[、,\n]/).map(x => x.trim()).filter(Boolean))
const masteryLabel = computed(() => {
  if (!item.value) return '待整理'
  if (item.value.masteryLevel >= 2) return '已掌握'
  if (item.value.masteryLevel === 1) return '复习中'
  return '新错题'
})
const mistakeStatusLabel = computed(() => {
  switch (item.value?.mistakeStatus) {
    case 'wrong_attempt':
      return '答错了'
    case 'not_attempted':
      return '未作答'
    case 'unknown':
      return '待判断'
    default:
      return '待判断'
  }
})
const notebookLabel = computed(() => item.value?.notebookName || '默认错题本')

onMounted(load)

async function load() {
  const items = await errorApi.getErrorItems()
  item.value = items.find(x => x.id === route.params.id) || null
  if (item.value) {
    form.value.questionText = item.value.questionText || ''
    form.value.answerText = item.value.answerText || ''
    form.value.analysis = item.value.analysis || ''
    form.value.mistakeAnalysis = item.value.mistakeAnalysis || ''
    form.value.userNotes = item.value.userNotes || ''
    form.value.knowledgePointsText = errorApi.parseKnowledgePoints(item.value.knowledgePoints).join('、')
  }
}

async function save() {
  if (!item.value || saving.value) return
  saving.value = true
  saveMessage.value = ''

  try {
    await errorApi.saveErrorItem({
      id: item.value.id,
      questionText: form.value.questionText,
      answerText: form.value.answerText,
      analysis: form.value.analysis,
      mistakeAnalysis: form.value.mistakeAnalysis,
      userNotes: form.value.userNotes,
      knowledgePoints: knowledgePoints.value,
    })
    await load()
    saveMessage.value = '已保存到本地错题卡'
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="app-page min-h-screen">
    <NavBar @back="router.push({ name: 'ErrorNotebook' })">
      <template #left>
        <div>
          <h1 class="text-sm font-black text-ink">错题详情</h1>
          <p class="mt-1 text-xs text-slate-400">{{ item?.syncStatus || 'local' }}</p>
        </div>
      </template>
      <template #right>
        <button class="ghost-button h-10 rounded-full px-4 text-xs font-black" :disabled="saving || !item" @click="save">
          <Save class="h-3.5 w-3.5" />
          {{ saving ? '保存中' : '保存' }}
        </button>
      </template>
    </NavBar>

    <main v-if="item" class="grid gap-4 px-4 pt-4">
      <section class="detail-hero soft-panel overflow-hidden p-4">
        <div class="grid gap-4">
          <img v-if="imageSrc" :src="imageSrc" class="max-h-80 w-full rounded-[24px] object-contain bg-white/90 shadow-[0_18px_38px_rgba(83,116,191,0.12)]" />

          <div class="grid gap-3">
            <div class="flex flex-wrap gap-2">
              <span class="detail-pill detail-pill-primary">{{ masteryLabel }}</span>
              <span class="detail-pill">{{ mistakeStatusLabel }}</span>
              <span class="detail-pill">{{ notebookLabel }}</span>
            </div>

            <div class="rounded-[24px] bg-white/82 p-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.94)]">
              <p class="section-kicker">题目理解</p>
              <textarea
                v-model="form.questionText"
                class="detail-textarea mt-3 min-h-34 text-base font-black leading-7 text-ink"
                placeholder="把题干整理成自己一眼能读懂的版本"
              />
            </div>

            <div class="grid grid-cols-2 gap-3">
              <div class="tiny-card flex items-center gap-3 p-3">
                <div class="icon-well bg-blue-50 text-blue-500">
                  <Clock3 class="h-4 w-4" />
                </div>
                <div class="min-w-0">
                  <p class="text-[11px] font-black uppercase tracking-[0.14em] text-slate-400">下次复习</p>
                  <p class="truncate text-sm font-bold text-ink">{{ item.nextReview }}</p>
                </div>
              </div>
              <div class="tiny-card flex items-center gap-3 p-3">
                <div class="icon-well bg-amber-50 text-amber-500">
                  <Target class="h-4 w-4" />
                </div>
                <div class="min-w-0">
                  <p class="text-[11px] font-black uppercase tracking-[0.14em] text-slate-400">复习强度</p>
                  <p class="truncate text-sm font-bold text-ink">Lv. {{ item.masteryLevel }} · {{ item.repetitions }} 次</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section class="soft-panel overflow-hidden p-0">
        <div class="detail-band detail-band-answer p-4">
          <div class="flex items-center gap-3">
            <div class="icon-well bg-white/22 text-white">
              <Sparkles class="h-4 w-4" />
            </div>
            <div>
              <p class="text-xs font-black uppercase tracking-[0.16em] text-white/74">解题路径</p>
              <h2 class="text-lg font-black text-white">答案和解析放在一起读</h2>
            </div>
          </div>
        </div>

        <div class="grid gap-4 p-4">
          <div class="detail-surface">
            <p class="mb-3 text-xs font-black uppercase tracking-[0.14em] text-blue-500">标准答案</p>
            <textarea
              v-model="form.answerText"
              class="detail-textarea min-h-20 text-sm leading-6 text-ink"
              placeholder="用最短路径写出标准答案"
            />
          </div>

          <div class="detail-surface">
            <div class="mb-3 flex items-center gap-2 text-slate-500">
              <Lightbulb class="h-4 w-4 text-amber-500" />
              <p class="text-xs font-black uppercase tracking-[0.14em]">为什么这么做</p>
            </div>
            <textarea
              v-model="form.analysis"
              class="detail-textarea min-h-32 text-sm leading-6 text-slate-700"
              placeholder="把关键思路、公式、判断步骤串成一段顺畅的解析"
            />
          </div>
        </div>
      </section>

      <section class="soft-panel overflow-hidden p-0">
        <div class="detail-band detail-band-mistake p-4">
          <div class="flex items-center gap-3">
            <div class="icon-well bg-white/20 text-white">
              <Brain class="h-4 w-4" />
            </div>
            <div>
              <p class="text-xs font-black uppercase tracking-[0.16em] text-white/74">纠错重点</p>
              <h2 class="text-lg font-black text-white">把错误原因和提醒放在一起</h2>
            </div>
          </div>
        </div>

        <div class="grid gap-4 p-4">
          <div v-if="item.wrongAnswerText" class="detail-surface">
            <p class="mb-3 text-xs font-black uppercase tracking-[0.14em] text-rose-500">当时写成了什么</p>
            <p class="whitespace-pre-wrap text-sm leading-6 text-ink">{{ item.wrongAnswerText }}</p>
          </div>

          <div class="detail-surface">
            <p class="mb-3 text-xs font-black uppercase tracking-[0.14em] text-rose-500">错因分析</p>
            <textarea
              v-model="form.mistakeAnalysis"
              class="detail-textarea min-h-26 text-sm leading-6 text-slate-700"
              placeholder="记录这次真正卡住的点：概念混淆、漏条件、粗心还是步骤断裂"
            />
          </div>

          <div class="detail-surface">
            <div class="mb-3 flex items-center gap-2 text-slate-500">
              <NotebookPen class="h-4 w-4 text-indigo-500" />
              <p class="text-xs font-black uppercase tracking-[0.14em]">我的提醒</p>
            </div>
            <textarea
              v-model="form.userNotes"
              class="detail-textarea min-h-24 text-sm leading-6 text-slate-700"
              placeholder="写一句下次看到这类题先做什么、先检查什么"
            />
          </div>
        </div>
      </section>

      <section class="soft-panel p-4">
        <div class="flex items-center gap-2 text-slate-500">
          <Tag class="h-4 w-4 text-blue-500" />
          <p class="text-xs font-black uppercase tracking-[0.14em]">知识点锚点</p>
        </div>
        <input
          v-model="form.knowledgePointsText"
          class="input-soft mt-3 h-12 w-full px-4 text-sm text-ink"
          placeholder="用 、 或逗号分隔，例如：一次函数、最值、分类讨论"
        />
        <div v-if="knowledgePoints.length" class="mt-3 flex flex-wrap gap-2">
          <span v-for="point in knowledgePoints" :key="point" class="detail-chip">
            {{ point }}
          </span>
        </div>
      </section>

      <p v-if="saveMessage" class="rounded-[22px] bg-emerald-50 px-4 py-3 text-sm font-bold text-emerald-600">
        {{ saveMessage }}
      </p>
    </main>

    <main v-else class="px-4 pt-4">
      <section class="soft-panel p-8 text-center">
        <p class="text-base font-black text-ink">这道错题还没找到</p>
        <p class="mt-2 text-sm text-slate-400">可能已经被删除，或者还没同步到本地。</p>
      </section>
    </main>
  </div>
</template>
