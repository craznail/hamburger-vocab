import { getCachedAudio, setCachedAudio } from './tts/cache'
import { getTtsSettings, makeTtsCacheKey } from './tts/config'
import { synthesizeCloudSpeech } from './tts/providers/cloud'
import { isNativeTtsAvailable, speakNative } from './tts/providers/native'
import { initAudioPlayer, playAudioBuffer } from './tts/player'

export { getTtsSettings, saveTtsSettings, TTS_PROVIDERS } from './tts/config'

const memoryAudioCache = new Map()
const pendingAudio = new Map()
const MAX_MEMORY_ITEMS = 24

export function initAudio() {
  initAudioPlayer().catch(() => {})
}

function rememberAudio(key, audio) {
  if (memoryAudioCache.has(key)) memoryAudioCache.delete(key)
  memoryAudioCache.set(key, audio)
  if (memoryAudioCache.size > MAX_MEMORY_ITEMS) {
    memoryAudioCache.delete(memoryAudioCache.keys().next().value)
  }
}

async function loadAudio(text, settings) {
  const cacheKey = makeTtsCacheKey(text, settings)
  const memoryHit = memoryAudioCache.get(cacheKey)
  if (memoryHit) return memoryHit
  if (pendingAudio.has(cacheKey)) return pendingAudio.get(cacheKey)

  const request = (async () => {
    let audio = await getCachedAudio(cacheKey)
    if (!audio) {
      audio = await synthesizeCloudSpeech(text, settings)
      setCachedAudio(cacheKey, audio)
    }
    rememberAudio(cacheKey, audio)
    return audio
  })()

  pendingAudio.set(cacheKey, request)
  try {
    return await request
  } finally {
    pendingAudio.delete(cacheKey)
  }
}

export async function prepareSpeech(text) {
  const value = text?.trim()
  if (!value) return
  await loadAudio(value, getTtsSettings())
}

export async function speakWord(text, options = {}) {
  const word = text?.trim()
  if (!word) {
    throw new Error('发音文本不能为空')
  }

  const settings = getTtsSettings()

  try {
    options.onStateChange?.('loading')
    const audio = await loadAudio(word, settings)

    if (options.signal?.aborted) {
      options.onStateChange?.('idle')
      return
    }

    await playAudioBuffer(audio.buffer, {
      ...options,
      contentType: audio.contentType
    })
  } catch (error) {
    if (options.signal?.aborted || error?.name === 'AbortError') {
      options.onStateChange?.('idle')
      return
    }
    if (isNativeTtsAvailable()) {
      return speakNative(word, options)
    }
    options.onStateChange?.('unavailable')
    throw error instanceof Error ? error : new Error(String(error))
  }
}
