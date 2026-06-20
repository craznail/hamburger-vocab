import { invoke } from '@tauri-apps/api/core'

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

export async function getErrorNotebooks(): Promise<ErrorNotebook[]> {
  return await invoke<ErrorNotebook[]>('get_error_notebooks')
}

export async function getErrorItems(notebookId: string | null = null): Promise<ErrorItem[]> {
  return await invoke<ErrorItem[]>('get_error_items', { notebookId })
}

export async function getDueErrorItems(): Promise<ErrorItem[]> {
  return await invoke<ErrorItem[]>('get_due_error_items')
}

export async function createErrorDraft(imageBase64: string, mimeType: string, notebookId: string | null = null): Promise<ErrorDraft> {
  return await invoke<ErrorDraft>('create_error_draft', {
    request: { imageBase64, mimeType, notebookId },
  })
}

export async function analyzeErrorDraft(id: string): Promise<ErrorItem> {
  return await invoke<ErrorItem>('analyze_error_draft', { id })
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

export function parseKnowledgePoints(value: string | null | undefined): string[] {
  if (!value) return []
  try {
    const parsed = JSON.parse(value)
    return Array.isArray(parsed) ? parsed : []
  } catch {
    return []
  }
}
