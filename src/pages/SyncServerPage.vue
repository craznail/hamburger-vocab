<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { Check, Cloud, Loader, LogOut, RefreshCw } from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
import * as authApi from '../api/auth'
import { useErrorNotebookStore } from '../stores/useErrorNotebookStore'
import { getAppSettings, saveAppSettings } from '../platform/appSettings'

const router = useRouter()
const notebookStore = useErrorNotebookStore()
const { auth, pendingCount } = storeToRefs(notebookStore)

const appSettings = reactive(getAppSettings())
const useMockData = computed(() => appSettings.errorNotebook?.enableMockDataFallback === true)

// 一次性迁移：读取旧的 localStorage key 回填到 appSettings
const legacyUrl = localStorage.getItem('wrongNotebookServerUrl')
if (legacyUrl && legacyUrl !== appSettings.sync.serverUrl) {
  appSettings.sync.serverUrl = legacyUrl
  saveAppSettings({ ...appSettings })
  localStorage.removeItem('wrongNotebookServerUrl')
}

const loginForm = reactive({
  serverUrl: appSettings.sync.serverUrl,
  email: '',
  password: '',
})
const message = ref('')
const loggingIn = ref(false)
const syncing = ref(false)

onMounted(() => {
  if (useMockData.value) return
  void notebookStore.ensureFresh()
})

async function login() {
  if (useMockData.value) {
    message.value = '当前为调试假数据模式，请到设置页关闭后再连接真实服务'
    return
  }
  message.value = ''
  loggingIn.value = true
  try {
    auth.value = await authApi.login(loginForm.serverUrl, loginForm.email, loginForm.password)
    appSettings.sync.serverUrl = loginForm.serverUrl
    saveAppSettings({ ...appSettings })
    message.value = '已登录远程服务端'
    notebookStore.invalidate()
    void notebookStore.refresh(true)
  } catch (e) {
    message.value = e instanceof Error ? e.message : String(e)
  } finally {
    loggingIn.value = false
  }
}

async function sync() {
  if (useMockData.value) {
    message.value = '当前为调试假数据模式，请到设置页关闭后再同步真实数据'
    return
  }
  message.value = ''
  syncing.value = true
  try {
    await notebookStore.syncRemote()
    message.value = '同步完成'
    notebookStore.invalidate()
    await notebookStore.refresh(true, false)
  } catch (e) {
    message.value = e instanceof Error ? e.message : String(e)
  } finally {
    syncing.value = false
  }
}

const displayAuth = computed(() => useMockData.value ? { loggedIn: false } : auth.value)

async function logout() {
  message.value = ''
  try {
    auth.value = await authApi.logout()
    message.value = '已退出登录，可重新输入账号连接'
  } catch (e) {
    message.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar @back="router.push({ name: 'Profile' })">
      <template #left>
        <h1 class="page-header-title">同步与服务端</h1>
      </template>
    </NavBar>

    <main class="flex-1 px-4 pb-8 pt-4">
      <section class="mb-5">
        <div class="mb-2 flex items-center justify-between px-1">
          <h2 class="text-xs font-bold text-slate-400">服务端连接</h2>
          <span class="text-[11px] font-semibold text-slate-400">
            {{ displayAuth.loggedIn ? '已连接' : '未连接' }}
          </span>
        </div>

        <div class="soft-panel overflow-hidden p-4">
          <!-- 已登录状态 -->
          <div v-if="displayAuth.loggedIn" class="space-y-3">
            <div class="flex items-center gap-3">
              <span class="grid h-8 w-8 place-items-center rounded-lg bg-blue-50 text-blue-500">
                <Cloud class="h-4 w-4" />
              </span>
              <div class="min-w-0 flex-1">
                <p class="text-sm font-bold text-ink">已连接远程服务端</p>
                <p class="mt-1 truncate text-xs text-slate-400">{{ loginForm.serverUrl || auth.serverUrl || '—' }}</p>
              </div>
              <span class="inline-flex items-center gap-1 rounded-full bg-green-50 px-3 py-1.5 text-[11px] font-bold text-green-600">
                <Check class="h-3.5 w-3.5" /> 在线
              </span>
            </div>
            <button
              class="flex h-10 w-full items-center justify-center gap-2 rounded-lg border border-slate-200 bg-white text-sm font-bold text-slate-500"
              @click="logout"
            >
              <LogOut class="h-4 w-4" />
              退出登录 / 切换账号
            </button>
          </div>

          <!-- 未登录：登录表单 -->
          <div v-else class="space-y-3">
            <label class="block">
              <span class="mb-1.5 block text-xs font-bold text-slate-500">服务端地址</span>
              <input
                v-model.trim="loginForm.serverUrl"
                placeholder="例如 http://localhost:3000"
                class="h-11 w-full rounded-lg border border-slate-200 bg-slate-50 px-3 text-sm text-ink outline-none focus:border-blue-300 focus:bg-white"
              />
            </label>
            <label class="block">
              <span class="mb-1.5 block text-xs font-bold text-slate-500">邮箱</span>
              <input
                v-model.trim="loginForm.email"
                autocomplete="off"
                placeholder="邮箱"
                class="h-11 w-full rounded-lg border border-slate-200 bg-slate-50 px-3 text-sm text-ink outline-none focus:border-blue-300 focus:bg-white"
              />
            </label>
            <label class="block">
              <span class="mb-1.5 block text-xs font-bold text-slate-500">密码</span>
              <input
                v-model="loginForm.password"
                type="password"
                autocomplete="off"
                placeholder="密码"
                class="h-11 w-full rounded-lg border border-slate-200 bg-slate-50 px-3 text-sm text-ink outline-none focus:border-blue-300 focus:bg-white"
              />
            </label>
            <button
              class="flex h-11 w-full items-center justify-center gap-2 rounded-lg bg-blue-600 text-sm font-bold text-white disabled:opacity-60"
              :disabled="loggingIn"
              @click="login"
            >
              <Loader v-if="loggingIn" class="h-4 w-4 animate-spin" />
              {{ loggingIn ? '连接中' : '登录并启用 AI 分析' }}
            </button>
            <p class="text-[11px] leading-5 text-slate-400">
              需要错题 AI 分析或跨设备同步时再连接即可。数据始终先保存在本地设备。
            </p>
          </div>
        </div>
      </section>

      <!-- 同步操作 -->
      <section v-if="displayAuth.loggedIn" class="mb-5">
        <div class="mb-2 flex items-center justify-between px-1">
          <h2 class="text-xs font-bold text-slate-400">手动同步</h2>
          <span class="text-[11px] font-semibold text-blue-500">{{ pendingCount }} 项待同步</span>
        </div>

        <div class="soft-panel overflow-hidden p-4">
          <p class="mb-3 text-xs leading-5 text-slate-400">
            将本地的错题、笔记与复习进度同步到当前服务端，并拉取远端最新数据。
          </p>
          <button
            class="flex h-11 w-full items-center justify-center gap-2 rounded-lg border border-blue-200 bg-blue-50 text-sm font-bold text-blue-600 disabled:opacity-60"
            :disabled="syncing"
            @click="sync"
          >
            <RefreshCw v-if="syncing" class="h-4 w-4 animate-spin" />
            <RefreshCw v-else class="h-4 w-4" />
            {{ syncing ? '同步中' : '立即同步' }}
          </button>
        </div>
      </section>

      <p v-if="message" class="mt-2 break-all rounded-lg bg-emerald-50 px-3 py-2 text-xs font-bold text-emerald-600">
        {{ message }}
      </p>
    </main>
  </div>
</template>
