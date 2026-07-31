import { invoke } from '@tauri-apps/api/core'
import { getAppSettings } from '../platform/appSettings'

export interface ErrorNotebook {
  id: string
  name: string
  createdAt: string
  itemCount: number
  dueCount: number
}

export interface ErrorItem {
  id: string
  remoteId?: string | null
  notebookId?: string | null
  notebookName?: string | null
  questionText?: string | null
  answerText?: string | null
  analysis?: string | null
  wrongAnswerText?: string | null
  mistakeAnalysis?: string | null
  mistakeStatus?: string | null
  knowledgePoints: string
  userNotes?: string | null
  masteryLevel: number
  ef: number
  interval: number
  repetitions: number
  nextReview: string
  syncStatus: string
  version: number
  createdAt: string
  updatedAt: string
  deletedAt?: string | null
  localImagePath?: string | null
  remoteImageUrl?: string | null
}

export interface ErrorDraft {
  id: string
  localImagePath: string
  sha256: string
  mimeType: string
  syncStatus: string
}

export interface ErrorSyncConflict {
  id: string
  localItemId: string
  remoteId?: string | null
  serverVersion: number
  reason: 'version_conflict' | 'validation_error' | 'not_found' | 'unknown'
  hasRemoteSnapshot: boolean
  createdAt: string
}

function normalizeServerUrl(value: string | null | undefined): string {
  return (value || '').trim().replace(/\/+$/, '')
}

export function resolveRemoteImageUrl(
  remoteImageUrl: string | null | undefined,
  serverUrl = getAppSettings().sync.serverUrl,
): string | null {
  if (!remoteImageUrl) return null
  if (
    remoteImageUrl.startsWith('http://') ||
    remoteImageUrl.startsWith('https://') ||
    remoteImageUrl.startsWith('data:') ||
    remoteImageUrl.startsWith('blob:')
  ) {
    return remoteImageUrl
  }

  const base = normalizeServerUrl(serverUrl)
  if (!base) return remoteImageUrl
  if (remoteImageUrl.startsWith('/')) return `${base}${remoteImageUrl}`
  return `${base}/${remoteImageUrl}`
}

export function normalizeErrorItem(item: ErrorItem, serverUrl?: string | null): ErrorItem {
  return {
    ...item,
    remoteImageUrl: resolveRemoteImageUrl(item.remoteImageUrl, serverUrl || undefined),
  }
}

export async function getErrorNotebooks(): Promise<ErrorNotebook[]> {
  return await invoke<ErrorNotebook[]>('get_error_notebooks')
}

export async function getErrorItems(notebookId: string | null = null): Promise<ErrorItem[]> {
  const items = await invoke<ErrorItem[]>('get_error_items', { notebookId })
  return items.map(item => normalizeErrorItem(item))
}

export async function getDueErrorItems(): Promise<ErrorItem[]> {
  const items = await invoke<ErrorItem[]>('get_due_error_items')
  return items.map(item => normalizeErrorItem(item))
}

export async function createErrorDraft(imageBase64: string, mimeType: string, notebookId: string | null = null): Promise<ErrorDraft> {
  return await invoke<ErrorDraft>('create_error_draft', {
    request: { imageBase64, mimeType, notebookId },
  })
}

export async function analyzeErrorDraft(id: string): Promise<ErrorItem> {
  const item = await invoke<ErrorItem>('analyze_error_draft', { id })
  return normalizeErrorItem(item)
}

export async function saveErrorItem(item: {
  id: string
  questionText?: string
  answerText?: string
  analysis?: string
  mistakeAnalysis?: string
  userNotes?: string
  knowledgePoints?: string[]
}): Promise<void> {
  await invoke('save_error_item', { request: item })
}

export async function rateErrorItem(id: string, quality: number, durationSeconds = 0): Promise<void> {
  await invoke('rate_error_item', {
    request: { id, quality, durationSeconds },
  })
}

export async function syncErrorItems(): Promise<unknown> {
  return await invoke('sync_error_items')
}

export async function getErrorSyncConflicts(): Promise<ErrorSyncConflict[]> {
  return await invoke<ErrorSyncConflict[]>('get_error_sync_conflicts')
}

export async function resolveErrorSyncConflictKeepLocal(localItemId: string): Promise<void> {
  await invoke('resolve_error_sync_conflict_keep_local', { localItemId })
}

export async function resolveErrorSyncConflictAcceptRemote(localItemId: string): Promise<void> {
  await invoke('resolve_error_sync_conflict_accept_remote', { localItemId })
}

export function parseKnowledgePoints(value: string | null | undefined): string[] {
  if (!value) return []
  try {
    const parsed = JSON.parse(value)
    return Array.isArray(parsed) ? parsed : []
  } catch {
    return []
  }
}
