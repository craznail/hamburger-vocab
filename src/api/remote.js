// Remote API layer (reserved for future server integration).
// When server-side features are needed, implement login, sync, etc. here.
// All remote calls go through Rust via invoke, not directly from JS.
//
// IMPORTANT: These functions call Rust commands that do NOT exist yet.
// They will fail at runtime if called. Implement the corresponding
// Rust commands in src-tauri/src/http/ before using these.

import { invoke } from '@tauri-apps/api/core'

const NOT_IMPLEMENTED = '服务端功能尚未实现，请等待后续版本更新'

/**
 * Login to remote server (reserved).
 */
export async function login(username, password) {
  try {
    return await invoke('login', { username, password })
  } catch {
    throw new Error(NOT_IMPLEMENTED)
  }
}

/**
 * Sync local progress to remote server (reserved).
 */
export async function syncProgress() {
  try {
    return await invoke('sync_progress')
  } catch {
    throw new Error(NOT_IMPLEMENTED)
  }
}

/**
 * Get current login status (reserved).
 */
export async function getAuthStatus() {
  try {
    return await invoke('get_auth_status')
  } catch {
    return { loggedIn: false }
  }
}
