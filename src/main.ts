import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './style.css'

type NativeSafeAreaBridge = {
  getTop: () => number
  getRight: () => number
  getBottom: () => number
  getLeft: () => number
}

const nativeWindow = window as typeof window & {
  __TAURI_INTERNALS__?: unknown
  NativeSafeArea?: NativeSafeAreaBridge
}

const isTauriAndroid =
  Boolean(nativeWindow.__TAURI_INTERNALS__) &&
  /Android/i.test(navigator.userAgent)

if (isTauriAndroid) {
  document.documentElement.classList.add('tauri-android')
}

function applyNativeSafeArea() {
  const safeArea = nativeWindow.NativeSafeArea
  if (!safeArea) return

  const root = document.documentElement
  root.style.setProperty('--safe-area-top', `${safeArea.getTop()}px`)
  root.style.setProperty('--safe-area-right', `${safeArea.getRight()}px`)
  root.style.setProperty('--safe-area-bottom', `${safeArea.getBottom()}px`)
  root.style.setProperty('--safe-area-left', `${safeArea.getLeft()}px`)
}

if (isTauriAndroid) {
  applyNativeSafeArea()
  window.addEventListener('resize', applyNativeSafeArea)
  document.addEventListener('visibilitychange', applyNativeSafeArea)
  requestAnimationFrame(applyNativeSafeArea)
}

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
