<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { AlertCircle, ArrowRight, Cloud, ImagePlus, Loader, NotebookPen, RefreshCw } from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
import BottomNav from '../components/BottomNav.vue'
import * as errorApi from '../api/errorItem'
import * as authApi from '../api/auth'
import { useErrorNotebookStore } from '../stores/useErrorNotebookStore'

const router = useRouter()
const notebookStore = useErrorNotebookStore()
const { notebooks, items, auth, loading, dueCount, pendingCount, initialized } = storeToRefs(notebookStore)
const syncMessage = ref('')
const loginForm = ref({
  serverUrl: localStorage.getItem('wrongNotebookServerUrl') || 'http://localhost:3000',
  email: '',
  password: '',
})

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
</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar :showBack="false">
      <template #left>
        <div>
          <h1 class="text-sm font-black text-ink">错题本</h1>
          <p class="mt-1 text-xs text-slate-400">图片分析、本地复习、云端同步</p>
        </div>
      </template>
    </NavBar>

    <main class="flex-1 px-4 pt-4">
      <section class="home-hero mb-4 p-5 text-white">
        <div class="flex items-start justify-between gap-4">
          <div>
            <div class="flex items-center gap-2 text-sm font-bold text-blue-100">
              <NotebookPen class="h-4 w-4" />
              今日错题复习
            </div>
            <div class="mt-4 text-6xl font-black leading-none">{{ dueCount }}</div>
            <p class="mt-2 text-sm text-blue-100/90">待复习错题 · {{ pendingCount }} 条待同步/待分析</p>
          </div>
          <button class="rounded-2xl bg-white/18 px-4 py-2 text-xs font-black" @click="sync">
            <RefreshCw class="mr-1 inline h-3.5 w-3.5" />
            同步
          </button>
        </div>
        <div class="mt-5 grid grid-cols-2 gap-3">
          <button class="primary-action text-sm" @click="router.push({ name: 'ErrorAdd' })">
            <ImagePlus class="h-4 w-4" />
            添加错题
          </button>
          <button class="primary-action text-sm" @click="router.push({ name: 'ErrorReview' })">
            开始复习
            <ArrowRight class="h-4 w-4" />
          </button>
        </div>
      </section>

      <section v-if="!auth.loggedIn" class="soft-panel mb-4 p-4">
        <div class="mb-3 flex items-center gap-2 text-sm font-black text-ink">
          <Cloud class="h-4 w-4 text-blue-500" />
          连接 wrong-notebook 服务端
        </div>
        <div class="grid gap-2">
          <input v-model="loginForm.serverUrl" class="input-soft h-11 px-3 text-sm" placeholder="服务端地址，例如 http://localhost:3000" />
          <input v-model="loginForm.email" class="input-soft h-11 px-3 text-sm" placeholder="邮箱" />
          <input v-model="loginForm.password" type="password" class="input-soft h-11 px-3 text-sm" placeholder="密码" />
          <button class="blue-gradient h-11 rounded-2xl text-sm font-bold text-white" @click="login">登录并启用 AI 分析</button>
        </div>
      </section>

      <p v-if="syncMessage" class="mb-3 rounded-2xl bg-white/80 px-4 py-3 text-xs font-semibold text-slate-500">
        {{ syncMessage }}
      </p>

      <section class="mb-4">
        <div class="mb-3 flex items-center justify-between px-1">
          <h2 class="text-base font-black text-ink">错题列表</h2>
          <Loader v-if="loading" class="h-4 w-4 animate-spin text-blue-400" />
        </div>

        <div v-if="items.length" class="grid gap-3">
          <button
            v-for="item in items"
            :key="item.id"
            class="soft-panel p-4 text-left"
            @click="router.push({ name: 'ErrorDetail', params: { id: item.id } })"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <h3 class="line-clamp-2 text-sm font-black text-ink">{{ item.questionText || '待 AI 分析的错题' }}</h3>
                <p class="mt-2 text-xs text-slate-400">下次复习：{{ item.nextReview }} · {{ item.syncStatus }}</p>
              </div>
              <span class="rounded-full bg-blue-50 px-2 py-1 text-[11px] font-bold text-blue-500">L{{ item.masteryLevel }}</span>
            </div>
          </button>
        </div>

        <div v-else-if="!initialized && loading" class="grid gap-3">
          <div v-for="placeholder in 3" :key="placeholder" class="soft-panel p-4 text-left">
            <div class="animate-pulse">
              <div class="h-4 w-3/4 rounded-full bg-blue-100"></div>
              <div class="mt-3 h-3 w-1/2 rounded-full bg-slate-100"></div>
            </div>
          </div>
        </div>

        <div v-else class="soft-panel p-8 text-center">
          <AlertCircle class="mx-auto mb-3 h-12 w-12 text-blue-200" />
          <p class="text-sm font-semibold text-slate-500">还没有错题</p>
          <p class="mt-1 text-xs muted">上传图片后可用 AI 自动分析</p>
        </div>
      </section>
    </main>
    <BottomNav />
  </div>
</template>
