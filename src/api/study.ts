import { invoke } from '@tauri-apps/api/core'
import type { RateResult } from '../types/generated/RateResult'
import type { LearningStats } from '../types/generated/LearningStats'

export async function rateCard(
  cardId: string,
  quality: number,
  durationSeconds?: number,
): Promise<RateResult> {
  return await invoke<RateResult>('rate_card', { cardId, quality, durationSeconds: durationSeconds ?? 0 })
}

export async function ratePracticeCard(
  cardId: string,
  quality: number,
  durationSeconds?: number,
): Promise<void> {
  await invoke('rate_practice_card', { cardId, quality, durationSeconds: durationSeconds ?? 0 })
}

export async function getLearningStats(): Promise<LearningStats> {
  return await invoke<LearningStats>('get_learning_stats')
}
