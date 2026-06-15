import { invoke } from '@tauri-apps/api/core'
import type { ParseResult } from '../types/generated/ParseResult'
import type { ImportFromTextResult } from '../types/generated/ImportFromTextResult'

export async function parseTxtContent(text: string): Promise<ParseResult> {
  return await invoke<ParseResult>('parse_txt_content', { text })
}

export async function importFromText(
  deckName: string,
  text: string,
): Promise<ImportFromTextResult> {
  return await invoke<ImportFromTextResult>('import_from_text', { deckName, text })
}
