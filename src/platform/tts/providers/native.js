export function isNativeTtsAvailable() {
  return typeof window !== 'undefined'
    && window.NativeTts
    && window.NativeTts.isAvailable
    && window.NativeTts.isAvailable()
}

export function speakNative(text, { onStateChange, signal, timeoutMs = 30000 } = {}) {
  return new Promise((resolve, reject) => {
    const tts = window.NativeTts
    if (!tts || !tts.isAvailable()) {
      onStateChange?.('unavailable')
      reject(new Error('原生 TTS 不可用'))
      return
    }

    let settled = false
    let pollTimer

    function cleanup() {
      settled = true
      clearTimeout(timeoutTimer)
      clearInterval(pollTimer)
      signal?.removeEventListener('abort', abort)
    }

    function abort() {
      if (settled) return
      tts.pause()
      cleanup()
      onStateChange?.('idle')
      resolve()
    }

    const timeoutTimer = setTimeout(() => {
      if (settled) return
      tts.pause()
      cleanup()
      onStateChange?.('unavailable')
      reject(new Error('原生 TTS 播放超时'))
    }, timeoutMs)

    if (signal?.aborted) {
      abort()
      return
    }
    signal?.addEventListener('abort', abort, { once: true })

    onStateChange?.('loading')
    tts.speak(text)
    onStateChange?.('playing')
    pollTimer = setInterval(() => {
      if (!settled && !tts.isSpeaking()) {
        cleanup()
        onStateChange?.('idle')
        resolve()
      }
    }, 200)
  })
}
