import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './style.css'

const isTauriAndroid =
  Boolean(window.__TAURI_INTERNALS__) &&
  /Android/i.test(navigator.userAgent)

if (isTauriAndroid) {
  document.documentElement.classList.add('tauri-android')
}

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
