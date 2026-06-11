// TTS platform adaptation
// Keeps the existing speech.js logic but wraps it in a platform-aware interface.
// TTS API calls will be migrated to Rust in a future step.

import { hasNativeTts, isMobile } from './index'

// ---------- IndexedDB audio cache ----------

const TTS_MODEL = 'qwen3-tts-flash'
const TTS_VOICE = 'Jennifer'

function cacheKey(word) {
  return `${TTS_MODEL}:${TTS_VOICE}:${word.toLowerCase().trim()}`
}

function openCache() {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open('tts-audio-cache', 1)
    req.onupgradeneeded = () => { req.result.createObjectStore('audio') }
    req.onsuccess = () => resolve(req.result)
    req.onerror = () => reject(req.error)
  })
}

async function getCachedBuffer(word) {
  try {
    const db = await openCache()
    return new Promise((resolve) => {
      const tx = db.transaction('audio', 'readonly')
      const req = tx.objectStore('audio').get(cacheKey(word))
      req.onsuccess = () => resolve(req.result?.buffer ?? null)
      req.onerror = () => resolve(null)
    })
  } catch { return null }
}

async function setCachedBuffer(word, buffer) {
  try {
    const db = await openCache()
    return new Promise((resolve) => {
      const tx = db.transaction('audio', 'readwrite')
      tx.objectStore('audio').put({ buffer, ts: Date.now() }, cacheKey(word))
      tx.oncomplete = () => resolve()
      tx.onerror = () => resolve()
    })
  } catch { /* silent */ }
}

const DASHSCOPE_API_URL = 'https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation'

function getApiKey() {
  return import.meta.env.VITE_DASHSCOPE_API_KEY || localStorage.getItem('dashscope_api_key') || ''
}

// ---------- Native Android TTS ----------

function speakNative(word, { onStateChange, signal, timeoutMs = 30000 } = {}) {
  return new Promise((resolve, reject) => {
    const tts = window.NativeTts
    if (!tts || !tts.isAvailable()) {
      onStateChange?.('unavailable')
      reject(new Error('原生 TTS 不可用'))
      return
    }

    onStateChange?.('loading')

    let settled = false
    let pollTimer = null

    function done() {
      settled = true
      if (pollTimer) { clearInterval(pollTimer); pollTimer = null }
    }

    const timeoutTimer = setTimeout(() => {
      if (!settled) {
        done()
        tts.pause()
        onStateChange?.('unavailable')
        reject(new Error('原生 TTS 播放超时'))
      }
    }, timeoutMs)

    function onAbort() {
      if (!settled) {
        done()
        clearTimeout(timeoutTimer)
        tts.pause()
        onStateChange?.('idle')
        resolve()
      }
    }
    if (signal?.aborted) {
      onAbort()
      return
    }
    signal?.addEventListener('abort', onAbort, { once: true })

    tts.speak(word)
    onStateChange?.('playing')

    pollTimer = setInterval(() => {
      if (settled) return
      if (!tts.isSpeaking()) {
        done()
        clearTimeout(timeoutTimer)
        signal?.removeEventListener('abort', onAbort)
        onStateChange?.('idle')
        resolve()
      }
    }, 200)
  })
}

// ---------- Web Audio API ----------

let audioContext = null

async function getAudioContext() {
  if (!audioContext) {
    audioContext = new (window.AudioContext || window.webkitAudioContext)()
  }
  if (audioContext.state === 'suspended') {
    await audioContext.resume()
  }
  return audioContext
}

export function initAudio() {
  getAudioContext().catch(() => {})
}

// ---------- Web TTS (DashScope API + AudioContext + IndexedDB cache) ----------

async function speakWeb(word, { onStateChange, signal, timeoutMs = 30000 } = {}) {
  let settled = false
  let timeoutTimer = null
  let sourceNode = null

  function done() {
    settled = true
    if (timeoutTimer) { clearTimeout(timeoutTimer); timeoutTimer = null }
    if (sourceNode) {
      try { sourceNode.stop() } catch (_) {}
      sourceNode = null
    }
  }

  const ctx = await getAudioContext()

  return new Promise((resolve, reject) => {
    getCachedBuffer(word).then(cached => {
      if (settled) return

      if (cached) {
        ctx.decodeAudioData(cached.slice(0)).then(audioBuffer => {
          if (settled) return
          playBuffer(audioBuffer)
        }).catch(() => {
          if (settled) return
          fetchFromApi()
        })
        return
      }

      fetchFromApi()
    }).catch(() => {
      if (settled) return
      fetchFromApi()
    })

    function fetchFromApi() {
      onStateChange?.('loading')

      timeoutTimer = setTimeout(() => {
        if (!settled) {
          done()
          onStateChange?.('unavailable')
          reject(new Error('阿里云 TTS 请求超时'))
        }
      }, timeoutMs)

      function onAbort() {
        if (!settled) {
          done()
          onStateChange?.('idle')
          resolve()
        }
      }
      if (signal?.aborted) {
        onAbort()
        done()
        resolve()
        return
      }
      signal?.addEventListener('abort', onAbort, { once: true })

      const apiKey = getApiKey()
      if (!apiKey) {
        done()
        onStateChange?.('unavailable')
        reject(new Error('阿里云 API Key 未配置'))
        return
      }

      function cleanupAbort() {
        signal?.removeEventListener('abort', onAbort)
      }

      fetch(DASHSCOPE_API_URL, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${apiKey}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          model: TTS_MODEL,
          input: { text: word, voice: TTS_VOICE, language_type: 'Auto' },
        }),
        signal,
      })
      .then(res => res.json())
      .then(data => {
        if (settled) return

        const audioUrl = data.output?.audio?.url || data.output?.results?.[0]?.url
        if (!audioUrl) {
          cleanupAbort()
          done()
          onStateChange?.('unavailable')
          reject(new Error('TTS 响应中未找到音频 URL'))
          return
        }

        fetch(audioUrl, { signal })
          .then(res => res.arrayBuffer())
          .then(arrayBuffer => {
            if (settled) return
            setCachedBuffer(word, arrayBuffer.slice(0))
            return ctx.decodeAudioData(arrayBuffer)
          })
          .then(audioBuffer => {
            if (settled) return
            playBuffer(audioBuffer)
          })
          .catch(err => {
            if (settled) return
            if (ctx.state === 'suspended') {
              ctx.resume().catch(() => {})
            }
            cleanupAbort()
            done()
            onStateChange?.('unavailable')
            reject(err)
          })
      })
      .catch(err => {
        if (settled) return
        cleanupAbort()
        done()
        onStateChange?.('unavailable')
        if (err.name === 'AbortError') {
          resolve()
        } else {
          reject(err)
        }
      })
    }

    function playBuffer(audioBuffer) {
      sourceNode = ctx.createBufferSource()
      sourceNode.buffer = audioBuffer

      const gainNode = ctx.createGain()
      gainNode.gain.value = 2.0
      sourceNode.connect(gainNode)
      gainNode.connect(ctx.destination)

      sourceNode.onended = () => {
        settled = true
        if (timeoutTimer) { clearTimeout(timeoutTimer); timeoutTimer = null }
        onStateChange?.('idle')
        resolve()
      }

      sourceNode.start(0)
      onStateChange?.('playing')
    }
  })
}

// ---------- Unified entry point ----------

export async function speakWord(word, options = {}) {
  try {
    return await speakWeb(word, options)
  } catch {
    if (hasNativeTts) {
      return speakNative(word, options)
    }
    options.onStateChange?.('unavailable')
    throw new Error('TTS 不可用：请配置阿里云 API Key 或使用 Android 设备')
  }
}
