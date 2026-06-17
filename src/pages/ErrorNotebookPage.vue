<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { convertFileSrc } from '@tauri-apps/api/core'
import {
  AlertCircle,
  ArrowRight,
  Cloud,
  ImagePlus,
  Loader,
  NotebookPen,
  RefreshCw,
  Sparkles,
  Target,
} from 'lucide-vue-next'
import BottomNav from '../components/BottomNav.vue'
import * as errorApi from '../api/errorItem'
import * as authApi from '../api/auth'
import { useErrorNotebookStore } from '../stores/useErrorNotebookStore'

const router = useRouter()
const notebookStore = useErrorNotebookStore()
const { items, auth, loading, dueCount, pendingCount, initialized } = storeToRefs(notebookStore)
const syncMessage = ref('')
const showLoginForm = ref(false)
const loginForm = ref({
  serverUrl: localStorage.getItem('wrongNotebookServerUrl') || 'http://localhost:3000',
  email: '',
  password: '',
})

const sortedItems = computed(() => [...items.value].sort((a, b) => String(b.updatedAt).localeCompare(String(a.updatedAt))))

onMounted(() => {
  void notebookStore.ensureFresh()
})

async function login() {
  syncMessage.value = ''
  auth.value = await authApi.login(loginForm.value.serverUrl, loginForm.value.email, loginForm.value.password)
  localStorage.setItem('wrongNotebookServerUrl', loginForm.value.serverUrl)
  syncMessage.value = '已登录远程服务端'
  notebookStore.invalidate()
  void notebookStore.refresh(true)
}

async function sync() {
  syncMessage.value = ''
  try {
    await errorApi.syncErrorItems()
    syncMessage.value = '同步完成'
    notebookStore.invalidate()
    await notebookStore.refresh(true)
  } catch (e) {
    syncMessage.value = e instanceof Error ? e.message : String(e)
  }
}

function getImageSrc(item: errorApi.ErrorItem) {
  if (item.localImagePath) return convertFileSrc(item.localImagePath)
  return item.remoteImageUrl || ''
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
        <p class="error-header-subtitle">总览更轻，列表更聚焦</p>
      </div>
      <button class="error-header-button" type="button" aria-label="同步错题" @click="sync">
        <RefreshCw class="h-5 w-5" />
      </button>
    </header>

    <main class="error-main">
      <section class="error-overview-card">
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
            <div class="error-overview-count">{{ dueCount }}</div>
            <p class="error-overview-count-label">待复习</p>

            <div class="error-overview-side">
              <span class="error-meta-pill">
                <Target class="h-4 w-4" />
                {{ items.length }} 道
              </span>
              <span class="error-meta-pill">
                <Sparkles class="h-4 w-4" />
                {{ pendingCount }} 待同步
              </span>
            </div>
          </div>
        </div>
      </section>

      <section v-if="!auth.loggedIn" class="error-login-card">
        <div class="error-login-head">
          <div class="error-card-head">
            <Cloud class="h-5 w-5 text-[#2f7cff]" />
            <div>
              <h2 class="error-section-title">连接服务端</h2>
              <p class="error-panel-hint">需要 AI 分析或同步时再连接就行</p>
            </div>
          </div>
          <button class="error-login-toggle" type="button" @click="showLoginForm = !showLoginForm">
            {{ showLoginForm ? '收起' : '连接' }}
          </button>
        </div>

        <div v-if="showLoginForm" class="error-form-grid">
          <input v-model="loginForm.serverUrl" class="error-input" placeholder="服务端地址，例如 http://localhost:3000" />
          <input v-model="loginForm.email" class="error-input" placeholder="邮箱" />
          <input v-model="loginForm.password" type="password" class="error-input" placeholder="密码" />
          <button class="error-submit-button" type="button" @click="login">登录并启用 AI 分析</button>
        </div>
      </section>

      <p v-if="syncMessage" class="error-message">{{ syncMessage }}</p>

      <section class="error-list-panel">
        <div class="error-list-head">
          <h2 class="error-section-title">最近错题</h2>
          <Loader v-if="loading" class="h-4 w-4 animate-spin text-blue-400" />
        </div>

        <div v-if="sortedItems.length" class="error-list">
          <button
            v-for="item in sortedItems"
            :key="item.id"
            class="error-list-item"
            type="button"
            @click="router.push({ name: 'ErrorDetail', params: { id: item.id } })"
          >
            <img v-if="getImageSrc(item)" :src="getImageSrc(item)" class="error-list-thumb" />
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

        <div v-else-if="!initialized && loading" class="error-list">
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
  background:
    radial-gradient(circle at top center, rgba(146, 187, 255, 0.18), transparent 28%),
    linear-gradient(180deg, #fbfdff 0%, #f3f7ff 30%, #edf3ff 100%);
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
  font-size: 1.1rem;
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
  padding: 1.05rem 1.5rem 7.5rem;
}

.error-overview-card,
.error-login-card,
.error-list-panel {
  overflow: hidden;
  border: 1px solid rgba(214, 227, 255, 0.95);
  border-radius: 2rem;
  background:
    radial-gradient(circle at top right, rgba(255, 255, 255, 0.52), transparent 30%),
    linear-gradient(180deg, #fefeff 0%, #f4f8ff 100%);
  box-shadow:
    0 20px 44px rgba(93, 123, 194, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.98);
}

.error-overview-card {
  padding: 1.1rem 1.1rem 1rem;
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
  color: #1d2e62;
  font-size: 4rem;
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
.error-secondary-button,
.error-submit-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.45rem;
  min-height: 2.8rem;
  border: 0;
  border-radius: 1.05rem;
  font-size: 0.9rem;
  font-weight: 900;
}

.error-primary-button,
.error-secondary-button {
  padding: 0 1.1rem;
}

.error-primary-button,
.error-submit-button {
  background: linear-gradient(135deg, #3e87ff 0%, #1f6eff 100%);
  color: white;
  box-shadow: 0 14px 26px rgba(31, 110, 255, 0.24);
}

.error-secondary-button {
  border: 1px solid rgba(205, 219, 244, 0.95);
  background: rgba(255, 255, 255, 0.9);
  color: #1d2e62;
}

.error-login-card {
  margin-top: 1rem;
  padding: 1rem 1.05rem;
}

.error-login-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
}

.error-card-head {
  display: flex;
  align-items: flex-start;
  gap: 0.55rem;
}

.error-login-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 1.95rem;
  padding: 0 0.78rem;
  border: 1px solid rgba(205, 219, 244, 0.95);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.9);
  color: #4f82df;
  font-size: 0.76rem;
  font-weight: 800;
}

.error-section-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 900;
  color: #1a2b57;
}

.error-panel-hint {
  margin: 0.18rem 0 0;
  color: #8ea0c6;
  font-size: 0.78rem;
}

.error-form-grid {
  display: grid;
  gap: 0.75rem;
  margin-top: 0.85rem;
}

.error-input {
  width: 100%;
  height: 2.9rem;
  border: 1px solid rgba(199, 216, 249, 0.9);
  border-radius: 1rem;
  background: rgba(255, 255, 255, 0.88);
  padding: 0 1rem;
  color: #1c2b61;
  outline: none;
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
  padding: 1.05rem 1.05rem 0.95rem;
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
  border-radius: 1.28rem;
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
  border-radius: 0.82rem;
  border: 2px solid rgba(255, 255, 255, 0.96);
  box-shadow: 0 8px 18px rgba(106, 127, 176, 0.1);
}

.error-list-thumb-placeholder {
  background: linear-gradient(180deg, #eef4ff 0%, #e4eeff 100%);
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
  .error-login-card,
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
