import { invoke } from '@tauri-apps/api/core'

/**
 * Rate a card with a single invoke.
 * SM2 computation, card update, and review logging are all handled in Rust.
 *
 * @param {string} cardId
 * @param {number} quality - 0 (forgot), 3 (hazy), 5 (mastered)
 * @returns {Promise<{ef: number, interval: number, repetitions: number, nextReview: string}>}
 */
export async function rateCard(cardId, quality, durationSeconds = 0) {
  return await invoke('rate_card', { cardId, quality, durationSeconds })
}

export async function getLearningStats() {
  return await invoke('get_learning_stats')
}
