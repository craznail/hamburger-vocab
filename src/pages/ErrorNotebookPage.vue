<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { convertFileSrc } from '@tauri-apps/api/core'
import {
  AlertCircle,
  ArrowRight,
  ImagePlus,
  Loader,
  NotebookPen,
  RefreshCw,
  Sparkles,
  Target,
} from 'lucide-vue-next'
import BottomNav from '../components/BottomNav.vue'
import * as errorApi from '../api/errorItem'
import { useErrorNotebookStore } from '../stores/useErrorNotebookStore'
import { getAppSettings } from '../platform/appSettings'

const router = useRouter()
const notebookStore = useErrorNotebookStore()
const { items, auth, loading, syncError, dueCount, pendingCount, initialized } = storeToRefs(notebookStore)
const syncMessage = ref('')
const failedImages = ref(new Set<string>())
const errorNotebookHeroArt = new URL('../assets/hero/error-notebook-hero-bg.png', import.meta.url).href
const appSettings = getAppSettings()
const useMockData = computed(() => appSettings.errorNotebook?.enableMockDataFallback === true)

const mockItems = computed<errorApi.ErrorItem[]>(() => ([
  {
    id: 'mock-error-1',
    remoteId: null,
    notebookId: 'mock-notebook',
    notebookName: '英语错题',
    questionText: '选择正确的时态填空：By the time we arrived, the lecture ____.',
    answerText: 'had started',
    analysis: 'by the time 引导过去时间点，主句常用过去完成时，表示在到达前已经发生。',
    wrongAnswerText: 'started',
    mistakeAnalysis: '容易把两个过去动作都写成一般过去时，没有拉开先后关系。',
    mistakeStatus: 'due',
    knowledgePoints: JSON.stringify(['过去完成时', '时态辨析']),
    userNotes: '总把 started 和 had started 混掉',
    masteryLevel: 2,
    ef: 2.36,
    interval: 1,
    repetitions: 2,
    nextReview: '今天',
    syncStatus: 'synced',
    version: 1,
    createdAt: '2026-06-18 20:12:00',
    updatedAt: '2026-06-19 09:21:00',
    deletedAt: null,
    localImagePath: null,
    remoteImageUrl: '',
  },
  {
    id: 'mock-error-2',
    remoteId: null,
    notebookId: 'mock-notebook',
    notebookName: '英语错题',
    questionText: '阅读理解定位错误：主旨题误选了细节项，需要回到首段和尾段抓中心句。',
    answerText: 'A',
    analysis: '主旨题先看篇章框架，不要被中间例子带偏。',
    wrongAnswerText: 'C',
    mistakeAnalysis: '做题时抓到了具体例子，但没有回扣作者态度和全文主线。',
    mistakeStatus: 'due',
    knowledgePoints: JSON.stringify(['阅读主旨题', '篇章结构']),
    userNotes: '先看首尾句，再看转折',
    masteryLevel: 1,
    ef: 2.18,
    interval: 1,
    repetitions: 1,
    nextReview: '今天',
    syncStatus: 'pending_sync',
    version: 1,
    createdAt: '2026-06-18 19:02:00',
    updatedAt: '2026-06-19 08:40:00',
    deletedAt: null,
    localImagePath: null,
    remoteImageUrl: '',
  },
  {
    id: 'mock-error-3',
    remoteId: null,
    notebookId: 'mock-notebook',
    notebookName: '数学错题',
    questionText: '数列求和题：裂项后中间项没有完全消掉，最后一项符号写反。',
    answerText: '11/12',
    analysis: '裂项后先写出前 3 项和后 3 项，再观察抵消关系，最后统一通分。',
    wrongAnswerText: '13/12',
    mistakeAnalysis: '步骤不完整导致最后整理时把负号看成正号。',
    mistakeStatus: 'learning',
    knowledgePoints: JSON.stringify(['裂项求和', '分式化简']),
    userNotes: '草稿必须把首尾项写全',
    masteryLevel: 3,
    ef: 2.52,
    interval: 3,
    repetitions: 3,
    nextReview: '明天',
    syncStatus: 'synced',
    version: 1,
    createdAt: '2026-06-17 17:46:00',
    updatedAt: '2026-06-19 07:12:00',
    deletedAt: null,
    localImagePath: null,
    remoteImageUrl: '',
  },
  {
    id: 'mock-error-4',
    remoteId: null,
    notebookId: 'mock-notebook',
    notebookName: '英语错题',
    questionText: '完形填空词义辨析：ignore / neglect / overlook 三个近义词没有区分语境。',
    answerText: 'overlook',
    analysis: 'overlook 更符合“疏忽、未注意到”的语境，ignore 偏主动无视。',
    wrongAnswerText: 'ignore',
    mistakeAnalysis: '只记了中文意思，没有结合语气和搭配辨析。',
    mistakeStatus: 'pending_analysis',
    knowledgePoints: JSON.stringify(['词义辨析', '完形填空']),
    userNotes: '近义词要看语气强弱',
    masteryLevel: 0,
    ef: 2.1,
    interval: 0,
    repetitions: 0,
    nextReview: '待分析',
    syncStatus: 'pending_analysis',
    version: 1,
    createdAt: '2026-06-19 11:05:00',
    updatedAt: '2026-06-19 11:05:00',
    deletedAt: null,
    localImagePath: null,
    remoteImageUrl: '',
  },
]))

const displayItems = computed(() => useMockData.value ? mockItems.value : items.value)
const displayAuth = computed(() => useMockData.value ? { loggedIn: false } : auth.value)
const displayLoading = computed(() => useMockData.value ? false : loading.value)
const displayInitialized = computed(() => useMockData.value ? true : initialized.value)
const displayDueCount = computed(() => {
  if (!useMockData.value) return dueCount.value
  return mockItems.value.filter(item => item.nextReview === '今天' || item.mistakeStatus === 'due').length
})
const displayPendingCount = computed(() => {
  if (!useMockData.value) return pendingCount.value
  return mockItems.value.filter(item => item.syncStatus !== 'synced').length
})

const sortedItems = computed(() => [...displayItems.value].sort((a, b) => String(b.updatedAt).localeCompare(String(a.updatedAt))))
const statusMessage = computed(() => syncMessage.value || syncError.value)

onMounted(() => {
  if (useMockData.value) return
  void notebookStore.ensureFresh()
})

async function sync() {
  if (useMockData.value) {
    syncMessage.value = '当前为调试假数据模式，请到设置页关闭后再同步真实数据'
    return
  }
  syncMessage.value = ''
  try {
    await notebookStore.syncRemote()
    syncMessage.value = '同步完成'
    notebookStore.invalidate()
    await notebookStore.refresh(true, false)
  } catch (e) {
    syncMessage.value = e instanceof Error ? e.message : String(e)
  }
}

function getImageSrc(item: errorApi.ErrorItem) {
  if (failedImages.value.has(item.id)) return ''
  if (item.localImagePath) return convertFileSrc(item.localImagePath)
  return item.remoteImageUrl || ''
}

function markImageFailed(id: string) {
  failedImages.value = new Set([...failedImages.value, id])
}

function getKnowledgePoints(item: errorApi.ErrorItem) {
  return errorApi.parseKnowledgePoints(item.knowledgePoints).slice(0, 2)
}

function getStatusText(item: errorApi.ErrorItem) {
  if (item.syncStatus === 'pending_analysis') return '待分析'
  if (item.syncStatus === 'pending_sync') return '待同步'
  if (item.syncStatus === 'synced') return '已同步'
  return item.syncStatus || '本地保存'
}
</script>

<template>
  <div class="app-page notebook-page min-h-screen">
    <header class="error-header header-safe-top">
      <div class="error-header-spacer" />
      <div class="text-center">
        <h1 class="error-header-title">错题本</h1>
      </div>
      <button class="error-header-button" type="button" aria-label="同步错题" @click="sync">
        <RefreshCw class="h-5 w-5" />
      </button>
    </header>

    <main class="error-main">
      <section class="error-overview-card error-overview-card-hero" :style="{ '--hero-image': `url(${errorNotebookHeroArt})` }">
        <div class="error-overview-top">
          <div class="error-overview-main">
            <p class="error-overview-kicker">
              <NotebookPen class="h-4 w-4" />
              今日待复习
            </p>
            <p class="error-overview-lead">先看真正要复习的题，再决定继续新增还是直接复习。</p>
            <p class="error-overview-text">把整理和复习都压缩在一个更轻的入口里。</p>

            <div class="error-overview-actions">
              <button class="error-primary-button" type="button" @click="router.push({ name: 'ErrorAdd' })">
                <ImagePlus class="h-4 w-4" />
                添加错题
              </button>
              <button class="error-secondary-button" type="button" @click="router.push({ name: 'ErrorReview' })">
                开始复习
                <ArrowRight class="h-4 w-4" />
              </button>
            </div>
          </div>

          <div class="error-overview-stat">
            <div class="error-overview-count">{{ displayDueCount }}</div>
            <p class="error-overview-count-label">待复习</p>

            <div class="error-overview-side">
              <span class="error-meta-pill">
                <Target class="h-4 w-4" />
                {{ displayItems.length }} 道
              </span>
              <span class="error-meta-pill">
                <Sparkles class="h-4 w-4" />
                {{ displayPendingCount }} 待同步
              </span>
            </div>
          </div>
        </div>
      </section>

      <section v-if="!displayAuth.loggedIn && !useMockData" class="error-connect-hint">
        <div class="error-connect-copy">
          <p class="error-connect-title">服务端未连接</p>
          <p class="error-connect-text">要使用 AI 分析或同步错题，请前往 我的-同步与服务端 连接。</p>
        </div>
        <button class="error-connect-action" type="button" @click="router.push({ name: 'SyncServer' })">
          去连接
        </button>
      </section>

      <p v-if="statusMessage" class="error-message">{{ statusMessage }}</p>

      <section class="error-list-panel">
        <div class="error-list-head">
          <h2 class="error-section-title">最近错题</h2>
          <Loader v-if="displayLoading" class="h-4 w-4 animate-spin text-blue-400" />
        </div>

        <div v-if="sortedItems.length" class="error-list">
          <button
            v-for="item in sortedItems"
            :key="item.id"
            class="error-list-item"
            type="button"
            @click="router.push({ name: 'ErrorDetail', params: { id: item.id } })"
          >
            <img v-if="getImageSrc(item)" :src="getImageSrc(item)" class="error-list-thumb" @error="markImageFailed(item.id)" />
            <div v-else class="error-list-thumb error-list-thumb-placeholder">
              <NotebookPen class="h-5 w-5" />
            </div>

            <div class="error-list-copy">
              <p class="error-list-title">{{ item.questionText || '待 AI 分析的错题' }}</p>

              <div v-if="getKnowledgePoints(item).length" class="error-tag-row">
                <span v-for="point in getKnowledgePoints(item)" :key="point" class="error-chip">
                  {{ point }}
                </span>
              </div>

              <div class="error-list-meta-row">
                <span class="error-level-pill">L{{ item.masteryLevel }}</span>
                <span class="error-list-meta-text">下次复习 {{ item.nextReview }}</span>
                <span class="error-list-status">{{ getStatusText(item) }}</span>
              </div>
            </div>
          </button>
        </div>

        <div v-else-if="!displayInitialized && displayLoading" class="error-list">
          <div v-for="placeholder in 3" :key="placeholder" class="error-list-item error-list-item-placeholder">
            <div class="animate-pulse w-full">
              <div class="h-4 w-2/3 rounded-full bg-blue-100"></div>
              <div class="mt-3 h-3 w-1/3 rounded-full bg-slate-100"></div>
            </div>
          </div>
        </div>

        <div v-else class="error-empty-card">
          <div class="error-empty-icon-wrap">
            <AlertCircle class="mx-auto h-12 w-12 text-blue-200" />
          </div>
          <p class="error-empty-title">还没有错题</p>
          <p class="error-empty-text">上传图片后会自动整理成更易复习的错题卡</p>
          <button class="error-empty-action" type="button" @click="router.push({ name: 'ErrorAdd' })">
            <ImagePlus class="h-4 w-4" />
            添加第一道错题
          </button>
        </div>
      </section>
    </main>

    <BottomNav />
  </div>
</template>

<style scoped>
.notebook-page {
  background: transparent;
}

.error-header {
  display: grid;
  grid-template-columns: 2.75rem 1fr 2.75rem;
  align-items: center;
  gap: 0.75rem;
  padding: 0.9rem 1.5rem 0;
}

.error-header-spacer {
  width: 2.75rem;
  height: 2.75rem;
}

.error-header-title {
  margin: 0;
  text-align: center;
  font-size: 0.92rem;
  font-weight: 900;
  color: #18274f;
}

.error-header-subtitle {
  margin: 0.2rem 0 0;
  text-align: center;
  font-size: 0.76rem;
  color: #8ea0c6;
}

.error-header-button {
  display: grid;
  width: 2.75rem;
  height: 2.75rem;
  place-items: center;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: #23345f;
}

.error-main {
  padding: 1.05rem 1rem 7.5rem;
}

.error-overview-card,
.error-list-panel,
.error-connect-hint {
  overflow: hidden;
  border: 1px solid rgba(214, 227, 255, 0.95);
  border-radius: 1.55rem;
  background:
    linear-gradient(rgba(74, 113, 170, 0.025) 1px, transparent 1px),
    linear-gradient(90deg, rgba(74, 113, 170, 0.025) 1px, transparent 1px),
    linear-gradient(180deg, rgba(255, 255, 255, 0.96) 0%, rgba(246, 249, 255, 0.96) 100%);
  background-size: 18px 18px, 18px 18px, auto;
  box-shadow:
    0 18px 38px rgba(73, 100, 158, 0.07),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.error-overview-card {
  padding: 1.18rem 1.08rem 1rem;
}

.error-overview-card-hero {
  background-image:
    linear-gradient(180deg, rgba(255, 255, 255, 0.72) 0%, rgba(248, 251, 255, 0.9) 100%),
    var(--hero-image);
  background-position: center;
  background-repeat: no-repeat;
  background-size: cover;
}

.error-overview-top {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 1rem 1.1rem;
}

.error-overview-main {
  min-width: 0;
}

.error-overview-kicker {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
  margin: 0;
  color: #4a7be0;
  font-size: 0.84rem;
  font-weight: 800;
}

.error-overview-lead {
  margin: 0.48rem 0 0;
  max-width: 19rem;
  color: #5f77ab;
  font-size: 1rem;
  line-height: 1.6;
  font-weight: 800;
}

.error-overview-text {
  margin: 0.28rem 0 0;
  max-width: 18rem;
  color: #7284ae;
  font-size: 0.8rem;
  line-height: 1.54;
}

.error-overview-stat {
  display: grid;
  justify-items: end;
  align-content: center;
}

.error-overview-count {
  color: #152752;
  font-size: 4.35rem;
  line-height: 0.92;
  font-weight: 900;
}

.error-overview-count-label {
  margin: 0.2rem 0 0;
  color: #8ea0c6;
  font-size: 0.84rem;
  font-weight: 700;
}

.error-overview-side {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
  justify-content: flex-end;
  margin-top: 0.7rem;
}

.error-meta-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  min-height: 1.82rem;
  padding: 0 0.68rem;
  border: 1px solid rgba(188, 212, 255, 0.9);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.9);
  color: #4f82df;
  font-size: 0.72rem;
  font-weight: 800;
}

.error-overview-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.6rem;
  margin-top: 0.88rem;
}

.error-primary-button,
.error-secondary-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.45rem;
  min-height: 2.8rem;
  border: 0;
  border-radius: 1.05rem;
  font-size: 0.9rem;
  font-weight: 900;
  padding: 0 1.1rem;
}

.error-primary-button {
  background: linear-gradient(135deg, #4a80ff 0%, #245dff 100%);
  color: white;
  box-shadow: 0 14px 26px rgba(31, 110, 255, 0.24);
}

.error-secondary-button {
  border: 1px solid rgba(205, 219, 244, 0.95);
  background: rgba(255, 255, 255, 0.9);
  color: #1d2e62;
}

.error-connect-hint {
  margin-top: 1rem;
  padding: 0.92rem 1.05rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
}

.error-connect-copy {
  min-width: 0;
}

.error-connect-title {
  margin: 0;
  font-size: 0.94rem;
  font-weight: 900;
  color: #1a2b57;
}

.error-connect-text {
  margin: 0.2rem 0 0;
  color: #8ea0c6;
  font-size: 0.78rem;
  line-height: 1.5;
}

.error-connect-action {
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 2.1rem;
  padding: 0 1rem;
  border: 0;
  border-radius: 999px;
  background: linear-gradient(135deg, #4a80ff 0%, #245dff 100%);
  color: white;
  font-size: 0.8rem;
  font-weight: 900;
  box-shadow: 0 12px 22px rgba(31, 110, 255, 0.2);
}

.error-section-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 900;
  color: #1a2b57;
}

.error-message {
  margin: 0.95rem 0 0;
  border-radius: 1rem;
  background: rgba(236, 251, 242, 0.98);
  padding: 0.85rem 1rem;
  color: #23a35d;
  font-size: 0.85rem;
  font-weight: 800;
}

.error-list-panel {
  margin-top: 1rem;
  padding: 1rem 0.85rem 0.9rem;
}

.error-list-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
}

.error-list {
  display: grid;
  gap: 0.75rem;
  margin-top: 0.95rem;
}

.error-list-item {
  display: grid;
  grid-template-columns: 2.95rem minmax(0, 1fr);
  align-items: start;
  gap: 0.62rem;
  width: 100%;
  border: 1px solid rgba(224, 232, 248, 0.95);
  border-radius: 1.18rem;
  background: rgba(255, 255, 255, 0.92);
  padding: 0.68rem;
  text-align: left;
}

.error-list-item-placeholder {
  display: block;
}

.error-list-thumb {
  display: grid;
  width: 2.95rem;
  height: 2.95rem;
  place-items: center;
  object-fit: cover;
  border-radius: 0.78rem;
  border: 2px solid rgba(255, 255, 255, 0.96);
  box-shadow: 0 8px 18px rgba(106, 127, 176, 0.1);
}

.error-list-thumb-placeholder {
  background:
    linear-gradient(rgba(80, 105, 150, 0.08) 1px, transparent 1px),
    linear-gradient(90deg, rgba(80, 105, 150, 0.06) 1px, transparent 1px),
    linear-gradient(180deg, #f6f8fb 0%, #e9eef8 100%);
  background-size: 8px 8px, 8px 8px, auto;
  color: #8aa5d9;
}

.error-list-copy {
  min-width: 0;
}

.error-list-title {
  margin: 0;
  color: #1b2c5a;
  font-size: 0.9rem;
  line-height: 1.58;
  font-weight: 900;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.error-tag-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.28rem;
  margin-top: 0.22rem;
}

.error-chip {
  display: inline-flex;
  align-items: center;
  min-height: 1.32rem;
  padding: 0 0.42rem;
  border: 1px solid rgba(188, 212, 255, 0.9);
  border-radius: 999px;
  background: rgba(246, 250, 255, 1);
  color: #5182de;
  font-size: 0.64rem;
  font-weight: 800;
}

.error-list-meta-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.32rem 0.42rem;
  margin-top: 0.34rem;
}

.error-list-meta-text {
  color: #8ea0c6;
  font-size: 0.72rem;
}

.error-list-status {
  color: #4e7fe0;
  font-size: 0.72rem;
  font-weight: 800;
}

.error-level-pill {
  display: inline-flex;
  align-items: center;
  min-height: 1.34rem;
  padding: 0 0.42rem;
  border-radius: 999px;
  background: rgba(231, 240, 255, 0.98);
  color: #2f7cff;
  font-size: 0.62rem;
  font-weight: 900;
}

.error-empty-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1.65rem 1.2rem 1.75rem;
  text-align: center;
}

.error-empty-icon-wrap {
  display: grid;
  width: 4.25rem;
  height: 4.25rem;
  place-items: center;
  border-radius: 999px;
  background: linear-gradient(180deg, rgba(241, 247, 255, 0.98) 0%, rgba(232, 241, 255, 0.98) 100%);
}

.error-empty-title {
  margin: 0.8rem 0 0;
  color: #1c2b61;
  font-size: 0.98rem;
  font-weight: 900;
}

.error-empty-text {
  margin: 0.35rem 0 0;
  color: #92a1c7;
  font-size: 0.84rem;
}

.error-empty-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  min-height: 2.65rem;
  margin-top: 0.95rem;
  padding: 0 1rem;
  border: 0;
  border-radius: 999px;
  background: linear-gradient(135deg, #3e87ff 0%, #1f6eff 100%);
  color: white;
  font-size: 0.84rem;
  font-weight: 900;
  box-shadow: 0 12px 22px rgba(31, 110, 255, 0.2);
}

@media (max-width: 720px) {
  .error-main {
    padding: 1rem 1rem 7.2rem;
  }

  .error-header {
    padding-right: 1rem;
    padding-left: 1rem;
  }

  .error-overview-card,
  .error-connect-hint,
  .error-list-panel {
    border-radius: 1.7rem;
  }

  .error-overview-card {
    padding: 1.1rem;
  }

  .error-overview-top {
    grid-template-columns: 1fr;
    gap: 0.7rem;
  }

  .error-overview-count {
    font-size: 3rem;
  }

  .error-overview-stat {
    justify-items: start;
  }

  .error-overview-side {
    width: 100%;
    justify-content: flex-start;
  }

  .error-overview-lead {
    font-size: 0.94rem;
  }

  .error-overview-actions {
    display: grid;
    grid-template-columns: 1fr 1fr;
  }

  .error-list-item {
    grid-template-columns: 2.85rem minmax(0, 1fr);
    gap: 0.58rem;
  }

  .error-list-thumb {
    width: 2.85rem;
    height: 2.85rem;
  }
}
</style>
