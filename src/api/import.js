import { invoke } from '@tauri-apps/api/core'

/**
 * Parse text content (format detection + parsing + validation).
 * All parsing logic runs in Rust.
 *
 * @param {string} text - Raw text content
 * @returns {Promise<{format: string, rows: Array, preview: Array, errors: Array, totalLines: number, validCount: number}>}
 */
export async function parseTxtContent(text) {
  return await invoke('parse_txt_content', { text })
}

/**
 * Import from raw text: parse + create deck + import cards, all in one invoke.
 *
 * @param {string} deckName - Name for the new deck
 * @param {string} text - Raw text content
 * @returns {Promise<{deckId: string, deckName: string, count: number, parseResult: object}>}
 */
export async function importFromText(deckName, text) {
  return await invoke('import_from_text', { deckName, text })
}
