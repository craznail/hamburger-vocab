import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  base: './',
  clearScreen: false,
  server: {
    host: '0.0.0.0',
    port: Number(process.env.HAMBURGER_DEV_PORT || 5173),
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  }
})
