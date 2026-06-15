import { spawn } from 'node:child_process'
import process from 'node:process'
import { fileURLToPath } from 'node:url'
import { findAvailablePort } from './dev-port.mjs'

const target = process.argv[2] || 'desktop'
const extraArgs = process.argv.slice(3)
const port = await findAvailablePort()
const devHost = process.env.TAURI_DEV_HOST || 'localhost'
const devUrl = `http://${devHost}:${port}`
const tauriCli = fileURLToPath(new URL('../node_modules/@tauri-apps/cli/tauri.js', import.meta.url))

const tauriArgs =
  target === 'android'
    // The Android build rewrites gen/android/app/src/main/assets/tauri.conf.json.
    // Watching src-tauri sees that generated file and causes an install loop.
    // Vite HMR remains active; restart this command after native Rust/Kotlin changes.
    ? ['android', 'dev', '--no-dev-server-wait', '--no-watch']
    : ['dev']

const configOverride = JSON.stringify({
  build: {
    devUrl
  }
})

console.log(`Starting ${target} dev server at ${devUrl}`)

const child = spawn(process.execPath, [tauriCli, ...tauriArgs, '--config', configOverride, ...extraArgs], {
  stdio: 'inherit',
  env: {
    ...process.env,
    HAMBURGER_DEV_PORT: String(port)
  }
})

child.on('error', (error) => {
  console.error(`Failed to start Tauri CLI: ${error.message}`)
  process.exit(1)
})

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal)
    return
  }

  process.exit(code ?? 0)
})
