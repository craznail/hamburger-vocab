import { spawn } from 'node:child_process'
import process from 'node:process'
import { fileURLToPath } from 'node:url'
import { findAvailablePort, DEFAULT_PORT } from './dev-port.mjs'

const port = Number(process.env.HAMBURGER_DEV_PORT || await findAvailablePort())
const viteCli = fileURLToPath(new URL('../node_modules/vite/bin/vite.js', import.meta.url))
const args = ['--host', '0.0.0.0', '--port', String(port)]

if (port !== DEFAULT_PORT) {
  console.log(`Port ${DEFAULT_PORT} is busy, using ${port} instead.`)
}

const child = spawn(process.execPath, [viteCli, ...args], {
  stdio: 'inherit',
  env: {
    ...process.env,
    HAMBURGER_DEV_PORT: String(port)
  }
})

child.on('error', (error) => {
  console.error(`Failed to start Vite CLI: ${error.message}`)
  process.exit(1)
})

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal)
    return
  }

  process.exit(code ?? 0)
})
