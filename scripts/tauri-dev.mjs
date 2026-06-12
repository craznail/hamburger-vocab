import { spawn } from 'node:child_process'
import process from 'node:process'
import { findAvailablePort } from './dev-port.mjs'

const target = process.argv[2] || 'desktop'
const extraArgs = process.argv.slice(3)
const port = await findAvailablePort()
const devHost = process.env.TAURI_DEV_HOST || 'localhost'
const devUrl = `http://${devHost}:${port}`
const npxCommand = process.platform === 'win32' ? 'npx.cmd' : 'npx'

const tauriArgs =
  target === 'android'
    ? ['tauri', 'android', 'dev', '--no-dev-server-wait']
    : ['tauri', 'dev']

const configOverride = JSON.stringify({
  build: {
    devUrl
  }
})

console.log(`Starting ${target} dev server at ${devUrl}`)

const child = spawn(npxCommand, [...tauriArgs, '--config', configOverride, ...extraArgs], {
  stdio: 'inherit',
  env: {
    ...process.env,
    HAMBURGER_DEV_PORT: String(port)
  }
})

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal)
    return
  }

  process.exit(code ?? 0)
})
