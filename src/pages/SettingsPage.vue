<script setup>
import { reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { Bell, Check, ChevronRight, Cloud, DatabaseBackup, Eye, EyeOff, HelpCircle, KeyRound, Loader, Mic2, Palette, Play, RotateCcw, Save, Settings as SettingsIcon, SlidersHorizontal } from 'lucide-vue-next'
import NavBar from '../components/NavBar.vue'
import { downloadDB } from '../api/card'
import { getTtsSettings, saveTtsSettings, speakWord, TTS_PROVIDERS } from '../platform/tts'

const router = useRouter()
const ttsSettings = reactive(getTtsSettings())
const showApiKey = ref(false)
const saveState = ref('idle')
const testState = ref('idle')
const testError = ref('')

const azureVoices = [
  { value: 'en-US-JennyNeural', label: 'Jenny · 美式女声' },
  { value: 'en-US-GuyNeural', label: 'Guy · 美式男声' },
  { value: 'en-GB-SoniaNeural', label: 'Sonia · 英式女声' },
  { value: 'en-GB-RyanNeural', label: 'Ryan · 英式男声' }
]

function saveSpeechSettings() {
  saveTtsSettings(ttsSettings)
  saveState.value = 'saved'
  setTimeout(() => { saveState.value = 'idle' }, 1600)
}

async function testSpeech() {
  saveSpeechSettings()
  testError.value = ''
  try {
    await speakWord('Welcome to Recall', {
      onStateChange: state => { testState.value = state }
    })
  } catch (error) {
    testState.value = 'unavailable'
    testError.value = error?.message || String(error)
  }
}

const groups = [
  {
    title: '学习设置',
    items: [
      { label: '学习设置', icon: SlidersHorizontal },
      { label: '复习设置', icon: RotateCcw },
      { label: '提醒设置', icon: Bell },
      { label: '外观设置', icon: Palette }
    ]
  },
  {
    title: '数据与同步',
    items: [
      { label: '备份与恢复', icon: DatabaseBackup, action: downloadDB },
      { label: '数据导出', icon: Cloud, action: downloadDB }
    ]
  },
  {
    title: '关于应用',
    items: [
      { label: '关于 Recall', icon: Eye, value: 'v0.3.0' },
      { label: '帮助与反馈', icon: HelpCircle }
    ]
  }
]
</script>

<template>
  <div class="app-page flex min-h-screen flex-col">
    <NavBar @back="router.push({ name: 'Profile' })">
      <template #left>
        <h1 class="text-xl font-black text-ink">设置</h1>
      </template>
    </NavBar>

    <main class="flex-1 px-5 pb-8">
      <section class="mb-5">
        <div class="mb-2 flex items-center justify-between px-1">
          <h2 class="text-xs font-bold text-slate-400">语音服务</h2>
          <span class="text-[11px] font-semibold text-blue-500">可随时切换</span>
        </div>

        <div class="soft-panel overflow-hidden rounded-2xl p-4">
          <div class="grid grid-cols-2 gap-2 rounded-xl bg-slate-100 p-1">
            <button
              v-for="provider in TTS_PROVIDERS"
              :key="provider.id"
              class="flex h-10 items-center justify-center gap-2 rounded-lg text-xs font-bold transition-all"
              :class="ttsSettings.provider === provider.id ? 'bg-white text-blue-600 shadow-sm' : 'text-slate-500'"
              @click="ttsSettings.provider = provider.id"
            >
              <Mic2 class="h-4 w-4" />
              {{ provider.label }}
            </button>
          </div>

          <div v-if="ttsSettings.provider === 'azure'" class="mt-4 space-y-3">
            <label class="block">
              <span class="mb-1.5 block text-xs font-bold text-slate-500">Speech Key</span>
              <div class="relative">
                <KeyRound class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-300" />
                <input
                  v-model.trim="ttsSettings.azure.apiKey"
                  :type="showApiKey ? 'text' : 'password'"
                  autocomplete="off"
                  placeholder="Azure Speech API Key"
                  class="h-11 w-full rounded-lg border border-slate-200 bg-slate-50 pl-9 pr-11 text-sm text-ink outline-none focus:border-blue-300 focus:bg-white"
                />
                <button
                  class="absolute right-0 top-0 flex h-11 w-11 items-center justify-center text-slate-400"
                  :title="showApiKey ? '隐藏密钥' : '显示密钥'"
                  @click="showApiKey = !showApiKey"
                >
                  <EyeOff v-if="showApiKey" class="h-4 w-4" />
                  <Eye v-else class="h-4 w-4" />
                </button>
              </div>
            </label>

            <div class="grid grid-cols-2 gap-3">
              <label class="block">
                <span class="mb-1.5 block text-xs font-bold text-slate-500">Region</span>
                <input
                  v-model.trim="ttsSettings.azure.region"
                  placeholder="eastus"
                  class="h-11 w-full rounded-lg border border-slate-200 bg-slate-50 px-3 text-sm text-ink outline-none focus:border-blue-300 focus:bg-white"
                />
              </label>
              <label class="block">
                <span class="mb-1.5 block text-xs font-bold text-slate-500">语速</span>
                <select
                  v-model="ttsSettings.azure.rate"
                  class="h-11 w-full rounded-lg border border-slate-200 bg-slate-50 px-3 text-sm text-ink outline-none focus:border-blue-300 focus:bg-white"
                >
                  <option value="-20%">较慢</option>
                  <option value="-10%">慢</option>
                  <option value="0%">正常</option>
                  <option value="10%">快</option>
                  <option value="20%">较快</option>
                </select>
              </label>
            </div>

            <label class="block">
              <span class="mb-1.5 block text-xs font-bold text-slate-500">音量增强</span>
              <select
                v-model="ttsSettings.azure.volume"
                class="h-11 w-full rounded-lg border border-slate-200 bg-slate-50 px-3 text-sm text-ink outline-none focus:border-blue-300 focus:bg-white"
              >
                <option value="+0%">原始音量</option>
                <option value="+20%">增强 20%</option>
                <option value="+35%">增强 35%（推荐）</option>
                <option value="+50%">增强 50%</option>
                <option value="+70%">增强 70%</option>
              </select>
            </label>

            <label class="block">
              <span class="mb-1.5 block text-xs font-bold text-slate-500">Voice</span>
              <select
                v-model="ttsSettings.azure.voice"
                class="h-11 w-full rounded-lg border border-slate-200 bg-slate-50 px-3 text-sm text-ink outline-none focus:border-blue-300 focus:bg-white"
              >
                <option v-for="voice in azureVoices" :key="voice.value" :value="voice.value">
                  {{ voice.label }}
                </option>
              </select>
            </label>
          </div>

          <div v-else class="mt-4 space-y-3">
            <label class="block">
              <span class="mb-1.5 block text-xs font-bold text-slate-500">DashScope API Key</span>
              <div class="relative">
                <KeyRound class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-300" />
                <input
                  v-model.trim="ttsSettings.aliyun.apiKey"
                  :type="showApiKey ? 'text' : 'password'"
                  autocomplete="off"
                  placeholder="sk-..."
                  class="h-11 w-full rounded-lg border border-slate-200 bg-slate-50 pl-9 pr-11 text-sm text-ink outline-none focus:border-blue-300 focus:bg-white"
                />
                <button
                  class="absolute right-0 top-0 flex h-11 w-11 items-center justify-center text-slate-400"
                  :title="showApiKey ? '隐藏密钥' : '显示密钥'"
                  @click="showApiKey = !showApiKey"
                >
                  <EyeOff v-if="showApiKey" class="h-4 w-4" />
                  <Eye v-else class="h-4 w-4" />
                </button>
              </div>
            </label>
            <div class="grid grid-cols-2 gap-3">
              <label class="block">
                <span class="mb-1.5 block text-xs font-bold text-slate-500">Model</span>
                <input
                  v-model.trim="ttsSettings.aliyun.model"
                  class="h-11 w-full rounded-lg border border-slate-200 bg-slate-50 px-3 text-sm text-ink outline-none focus:border-blue-300 focus:bg-white"
                />
              </label>
              <label class="block">
                <span class="mb-1.5 block text-xs font-bold text-slate-500">Voice</span>
                <input
                  v-model.trim="ttsSettings.aliyun.voice"
                  class="h-11 w-full rounded-lg border border-slate-200 bg-slate-50 px-3 text-sm text-ink outline-none focus:border-blue-300 focus:bg-white"
                />
              </label>
            </div>
          </div>

          <div class="mt-4 grid grid-cols-2 gap-3">
            <button
              class="flex h-11 items-center justify-center gap-2 rounded-lg border border-blue-200 bg-blue-50 text-sm font-bold text-blue-600"
              :disabled="testState === 'loading' || testState === 'playing'"
              @click="testSpeech"
            >
              <Loader v-if="testState === 'loading'" class="h-4 w-4 animate-spin" />
              <Play v-else class="h-4 w-4" />
              {{ testState === 'playing' ? '播放中' : '试听' }}
            </button>
            <button
              class="flex h-11 items-center justify-center gap-2 rounded-lg bg-blue-600 text-sm font-bold text-white"
              @click="saveSpeechSettings"
            >
              <Check v-if="saveState === 'saved'" class="h-4 w-4" />
              <Save v-else class="h-4 w-4" />
              {{ saveState === 'saved' ? '已保存' : '保存配置' }}
            </button>
          </div>
          <p v-if="testError" class="mt-3 break-all text-xs text-red-500">{{ testError }}</p>
          <p class="mt-3 text-[11px] leading-5 text-slate-400">
            云端失败时会自动降级到 Android 系统语音。不同服务和声音使用独立缓存。
          </p>
        </div>
      </section>

      <section v-for="group in groups" :key="group.title" class="mb-5">
        <h2 class="mb-2 px-1 text-xs font-bold text-slate-400">{{ group.title }}</h2>
        <div class="soft-panel overflow-hidden rounded-2xl">
          <button v-for="item in group.items" :key="item.label" class="flex w-full items-center gap-3 border-b border-blue-50 px-4 py-3 text-left last:border-b-0" @click="item.action?.()">
            <span class="grid h-8 w-8 place-items-center rounded-lg bg-blue-50 text-blue-500">
              <component :is="item.icon || SettingsIcon" class="h-4 w-4" />
            </span>
            <span class="flex-1 text-sm font-bold text-ink">{{ item.label }}</span>
            <span v-if="item.value" class="text-xs text-slate-400">{{ item.value }}</span>
            <ChevronRight class="h-4 w-4 text-slate-300" />
          </button>
        </div>
      </section>
    </main>
  </div>
</template>
