// File picker platform adaptation
import { isAndroid, hasNativeFileResolver } from './index'

/**
 * Open a file picker dialog and return the selected file's path and name.
 * Handles platform differences (desktop vs Android content:// URIs).
 */
export async function pickFile(options = {}) {
  const { open } = await import('@tauri-apps/plugin-dialog')

  const selected = await open({
    multiple: false,
    filters: options.filters || [{ name: 'Text Files', extensions: ['txt'] }]
  })

  if (!selected) return null

  const filePath = typeof selected === 'string' ? selected : selected.path
  const fileName = await resolveFileName(filePath)

  return { path: filePath, name: fileName }
}

/**
 * Resolve display name from file path.
 * Handles Android content:// URIs and desktop paths.
 */
async function resolveFileName(filePath) {
  let fileName

  // 1) Android: query ContentResolver directly via JS bridge
  if (hasNativeFileResolver) {
    fileName = window.NativeFileResolver.getDisplayName(filePath)
  }

  // 2) Fallback: try stat() (works for content:// URIs)
  if (!fileName) {
    try {
      const { stat } = await import('@tauri-apps/plugin-fs')
      const info = await stat(filePath)
      fileName = info.name
    } catch { /* ignore */ }
  }

  // 3) Last resort: extract last path segment
  if (!fileName) {
    fileName = filePath.split(/[/\\]/).pop()
  }

  // Guard: if name is still a URI/ID (has : or URL encoding), use timestamp
  if (!fileName || fileName.includes(':') || /%[0-9A-Fa-f]{2}/.test(fileName)) {
    fileName = String(new Date().getFullYear()) + '.txt'
  }

  return fileName
}

/**
 * Read text file content.
 * Uses @tauri-apps/plugin-fs which handles Android content:// URIs correctly.
 * Falls back to Rust-side reading for desktop paths.
 */
export async function readTxtFile(path) {
  const TIMEOUT_MS = 15000
  const timeout = new Promise((_, reject) =>
    setTimeout(() => reject(new Error(`读取文件超时 (${path.substring(0, 50)}...)`)), TIMEOUT_MS)
  )

  // On Android, path is a content:// URI — must use fs plugin
  if (isAndroid || path.startsWith('content://')) {
    const { readTextFile } = await import('@tauri-apps/plugin-fs')
    return await Promise.race([readTextFile(path), timeout])
  }

  // Desktop: use Rust-side reading
  const { invoke } = await import('@tauri-apps/api/core')
  return await Promise.race([
    invoke('read_txt_content', { path }),
    timeout
  ])
}
