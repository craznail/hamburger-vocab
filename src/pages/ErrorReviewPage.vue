<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { convertFileSrc } from '@tauri-apps/api/core'
import {
  Brain,
  CheckCircle,
  Clock3,
  Lightbulb,
  NotebookPen,
  RotateCcw,
  Sparkles,
  Tag,
  Target,
} from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
import * as errorApi from '../api/errorItem'

const router = useRouter()
const items = ref<errorApi.ErrorItem[]>([])
const index = ref(0)
const revealed = ref(false)
const startedAt = ref(Date.now())
const done = ref(false)

const current = computed(() => items.value[index.value] || null)
const imageSrc = computed(() => current.value?.localImagePath ? convertFileSrc(current.value.localImagePath) : current.value?.remoteImageUrl || '')
const progressLabel = computed(() => done.value ? '完成' : `${index.value + 1} / ${items.value.length}`)
const masteryLabel = computed(() => {
  if (!current.value) return '待复习'
  if (current.value.masteryLevel >= 2) return '已掌握'
  if (current.value.masteryLevel === 1) return '复习中'
  return '新错题'
})
const mistakeStatusLabel = computed(() => {
  switch (current.value?.mistakeStatus) {
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
const knowledgePoints = computed(() => current.value ? errorApi.parseKnowledgePoints(current.value.knowledgePoints) : [])

onMounted(async () => {
  items.value = await errorApi.getDueErrorItems()
  if (items.value.length === 0) done.value = true
})

async function rate(quality: number) {
  if (!current.value) return
  const seconds = Math.max(1, Math.round((Date.now() - startedAt.value) / 1000))
  await errorApi.rateErrorItem(current.value.id, quality, seconds)
  if (index.value < items.value.length - 1) {
    index.value++
    revealed.value = false
    startedAt.value = Date.now()
  } else {
    done.value = true
  }
}
</script>

<template>
  <div class="app-page min-h-screen">
    <NavBar @back="router.push({ name: 'ErrorNotebook' })">
      <template #left>
        <div>
          <h1 class="text-sm font-black text-ink">错题复习</h1>
          <p class="mt-1 text-xs text-slate-400">{{ progressLabel }}</p>
        </div>
      </template>
    </NavBar>

    <main class="grid gap-4 px-4 pt-4">
      <section v-if="done" class="soft-panel overflow-hidden p-0">
        <div class="detail-band detail-band-review p-5 text-white">
          <CheckCircle class="mb-4 h-14 w-14 text-white/90" />
          <h2 class="text-2xl font-black">本轮错题复习完成</h2>
          <p class="mt-2 max-w-[18rem] text-sm leading-6 text-blue-100/90">
            这次一共完成 {{ items.length }} 道题。趁记忆还新，回到错题本挑一题再补一句提醒会更稳。
          </p>
        </div>

        <div class="grid gap-4 p-4">
          <div class="grid grid-cols-2 gap-3">
            <div class="tiny-card flex items-center gap-3 p-3">
              <div class="icon-well bg-green-50 text-green-500">
                <Target class="h-4 w-4" />
              </div>
              <div>
                <p class="text-[11px] font-black uppercase tracking-[0.14em] text-slate-400">完成题数</p>
                <p class="text-sm font-bold text-ink">{{ items.length }} 道</p>
              </div>
            </div>
            <div class="tiny-card flex items-center gap-3 p-3">
              <div class="icon-well bg-blue-50 text-blue-500">
                <Clock3 class="h-4 w-4" />
              </div>
              <div>
                <p class="text-[11px] font-black uppercase tracking-[0.14em] text-slate-400">当前状态</p>
                <p class="text-sm font-bold text-ink">今日已清空</p>
              </div>
            </div>
          </div>

          <button class="blue-gradient h-12 rounded-2xl text-sm font-bold text-white" @click="router.push({ name: 'ErrorNotebook' })">
            返回错题本
          </button>
        </div>
      </section>

      <template v-else-if="current">
        <section class="detail-band detail-band-review soft-panel overflow-hidden p-5 text-white">
          <div class="relative z-10 grid gap-4">
            <div class="flex flex-wrap items-center justify-between gap-3">
              <div>
                <p class="text-xs font-black uppercase tracking-[0.16em] text-blue-100/80">回忆模式</p>
                <h2 class="mt-1 text-2xl font-black leading-tight">先自己想，再看答案</h2>
              </div>
              <div class="flex flex-wrap gap-2">
                <span class="detail-pill bg-white/18 text-white border-white/14">{{ masteryLabel }}</span>
                <span class="detail-pill bg-white/18 text-white border-white/14">{{ mistakeStatusLabel }}</span>
              </div>
            </div>

            <div class="rounded-[24px] bg-white/14 p-4 backdrop-blur-sm">
              <p class="mb-2 text-xs font-black uppercase tracking-[0.14em] text-blue-100/80">当前题目</p>
              <p class="text-sm leading-6 text-blue-50">
                {{ revealed ? '已展开答案，读完后直接给自己一个掌握评分。' : '先根据题干和图片在脑中过一遍步骤，再点按钮展开。' }}
              </p>
            </div>
          </div>
        </section>

        <section class="detail-hero soft-panel overflow-hidden p-4">
          <div class="grid gap-4">
            <img v-if="imageSrc" :src="imageSrc" class="max-h-80 w-full rounded-[24px] object-contain bg-white/92 shadow-[0_18px_38px_rgba(83,116,191,0.12)]" />

            <div class="detail-surface">
              <p class="section-kicker">题目理解</p>
              <h3 class="mt-3 whitespace-pre-wrap text-base font-black leading-7 text-ink">
                {{ current.questionText || '这道题还没有题干，先根据图片回忆核心条件。' }}
              </h3>
            </div>

            <div class="grid grid-cols-2 gap-3">
              <div class="tiny-card flex items-center gap-3 p-3">
                <div class="icon-well bg-blue-50 text-blue-500">
                  <Clock3 class="h-4 w-4" />
                </div>
                <div class="min-w-0">
                  <p class="text-[11px] font-black uppercase tracking-[0.14em] text-slate-400">下次复习</p>
                  <p class="truncate text-sm font-bold text-ink">{{ current.nextReview }}</p>
                </div>
              </div>
              <div class="tiny-card flex items-center gap-3 p-3">
                <div class="icon-well bg-amber-50 text-amber-500">
                  <Target class="h-4 w-4" />
                </div>
                <div class="min-w-0">
                  <p class="text-[11px] font-black uppercase tracking-[0.14em] text-slate-400">复习强度</p>
                  <p class="truncate text-sm font-bold text-ink">Lv. {{ current.masteryLevel }} · {{ current.repetitions }} 次</p>
                </div>
              </div>
            </div>

            <button
              v-if="!revealed"
              class="blue-gradient flex h-12 w-full items-center justify-center gap-2 rounded-2xl text-sm font-bold text-white shadow-[0_18px_28px_rgba(53,100,255,0.22)]"
              @click="revealed = true"
            >
              <RotateCcw class="h-4 w-4" />
              我想好了，展开答案
            </button>
          </div>
        </section>

        <template v-if="revealed">
          <section class="soft-panel overflow-hidden p-0">
            <div class="detail-band detail-band-answer p-4">
              <div class="flex items-center gap-3">
                <div class="icon-well bg-white/22 text-white">
                  <Sparkles class="h-4 w-4" />
                </div>
                <div>
                  <p class="text-xs font-black uppercase tracking-[0.16em] text-white/74">解题路径</p>
                  <h2 class="text-lg font-black text-white">先对答案，再读思路</h2>
                </div>
              </div>
            </div>

            <div class="grid gap-4 p-4">
              <div class="detail-surface">
                <p class="mb-3 text-xs font-black uppercase tracking-[0.14em] text-blue-500">标准答案</p>
                <p class="whitespace-pre-wrap text-sm leading-6 text-ink">{{ current.answerText || '暂无答案' }}</p>
              </div>

              <div class="detail-surface">
                <div class="mb-3 flex items-center gap-2 text-slate-500">
                  <Lightbulb class="h-4 w-4 text-amber-500" />
                  <p class="text-xs font-black uppercase tracking-[0.14em]">关键解析</p>
                </div>
                <p class="whitespace-pre-wrap text-sm leading-6 text-slate-700">{{ current.analysis || '暂无解析' }}</p>
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
                  <h2 class="text-lg font-black text-white">把这次容易再错的点记牢</h2>
                </div>
              </div>
            </div>

            <div class="grid gap-4 p-4">
              <div v-if="current.wrongAnswerText" class="detail-surface">
                <p class="mb-3 text-xs font-black uppercase tracking-[0.14em] text-rose-500">你当时写成了什么</p>
                <p class="whitespace-pre-wrap text-sm leading-6 text-ink">{{ current.wrongAnswerText }}</p>
              </div>

              <div v-if="current.mistakeAnalysis" class="detail-surface">
                <p class="mb-3 text-xs font-black uppercase tracking-[0.14em] text-rose-500">错因分析</p>
                <p class="whitespace-pre-wrap text-sm leading-6 text-slate-700">{{ current.mistakeAnalysis }}</p>
              </div>

              <div v-if="current.userNotes" class="detail-surface">
                <div class="mb-3 flex items-center gap-2 text-slate-500">
                  <NotebookPen class="h-4 w-4 text-indigo-500" />
                  <p class="text-xs font-black uppercase tracking-[0.14em]">我的提醒</p>
                </div>
                <p class="whitespace-pre-wrap text-sm leading-6 text-slate-700">{{ current.userNotes }}</p>
              </div>
            </div>
          </section>

          <section v-if="knowledgePoints.length" class="soft-panel p-4">
            <div class="flex items-center gap-2 text-slate-500">
              <Tag class="h-4 w-4 text-blue-500" />
              <p class="text-xs font-black uppercase tracking-[0.14em]">知识点锚点</p>
            </div>
            <div class="mt-3 flex flex-wrap gap-2">
              <span v-for="point in knowledgePoints" :key="point" class="detail-chip">
                {{ point }}
              </span>
            </div>
          </section>

          <section class="soft-panel p-4">
            <p class="text-xs font-black uppercase tracking-[0.14em] text-slate-400">这次掌握得怎么样</p>
            <div class="mt-3 grid grid-cols-3 gap-3">
              <button class="red-gradient rounded-[22px] px-3 py-4 text-left text-white shadow-[0_16px_24px_rgba(255,95,89,0.2)]" @click="rate(0)">
                <p class="text-sm font-black">忘了</p>
                <p class="mt-1 text-[11px] leading-5 text-white/82">基本没想起来</p>
              </button>
              <button class="warm-gradient rounded-[22px] px-3 py-4 text-left text-white shadow-[0_16px_24px_rgba(255,179,56,0.22)]" @click="rate(3)">
                <p class="text-sm font-black">模糊</p>
                <p class="mt-1 text-[11px] leading-5 text-white/82">能做一半，还不稳</p>
              </button>
              <button class="green-gradient rounded-[22px] px-3 py-4 text-left text-white shadow-[0_16px_24px_rgba(78,212,93,0.2)]" @click="rate(5)">
                <p class="text-sm font-black">掌握</p>
                <p class="mt-1 text-[11px] leading-5 text-white/82">思路清楚，能独立做</p>
              </button>
            </div>
          </section>
        </template>
      </template>
    </main>
  </div>
</template>
