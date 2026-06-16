import { invoke } from '@tauri-apps/api/core'

export interface AuthStatus {
  loggedIn: boolean
  serverUrl?: string | null
  user?: unknown
}

export async function login(serverUrl: string, email: string, password: string): Promise<AuthStatus> {
  return await invoke<AuthStatus>('mobile_login', {
    request: { serverUrl, email, password },
  })
}

export async function getAuthStatus(): Promise<AuthStatus> {
  return await invoke<AuthStatus>('get_auth_status')
}
