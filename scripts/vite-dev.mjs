import { spawn } from 'node:child_process'
import process from 'node:process'
import { findAvailablePort, DEFAULT_PORT } from './dev-port.mjs'

const port = Number(process.env.HAMBURGER_DEV_PORT || await findAvailablePort())
const npxCommand = process.platform === 'win32' ? 'npx.cmd' : 'npx'
const args = ['vite', '--host', '0.0.0.0', '--port', String(port)]

if (port !== DEFAULT_PORT) {
  console.log(`Port ${DEFAULT_PORT} is busy, using ${port} instead.`)
}

const child = spawn(npxCommand, args, {
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
