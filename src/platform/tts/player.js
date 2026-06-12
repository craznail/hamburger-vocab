let audioContext = null
const decodedAudioCache = new WeakMap()

export async function initAudioPlayer() {
  if (!audioContext) {
    const AudioContext = window.AudioContext || window.webkitAudioContext
    if (!AudioContext) throw new Error('当前设备不支持音频播放')
    audioContext = new AudioContext()
  }
  if (audioContext.state === 'suspended') {
    await audioContext.resume()
  }
  return audioContext
}

export async function playAudioBuffer(buffer, {
  onStateChange,
  signal,
  timeoutMs = 30000
} = {}) {
  const context = await initAudioPlayer()
  let decoded = decodedAudioCache.get(buffer)
  if (!decoded) {
    decoded = await context.decodeAudioData(buffer.slice(0))
    decodedAudioCache.set(buffer, decoded)
  }

  return new Promise((resolve, reject) => {
    let settled = false
    const source = context.createBufferSource()
    const gain = context.createGain()
    const compressor = context.createDynamicsCompressor()
    source.buffer = decoded
    gain.gain.value = 1.75
    compressor.threshold.value = -18
    compressor.knee.value = 16
    compressor.ratio.value = 4
    compressor.attack.value = 0.003
    compressor.release.value = 0.2
    source.connect(gain)
    gain.connect(compressor)
    compressor.connect(context.destination)

    function finish(state = 'idle') {
      if (settled) return
      settled = true
      clearTimeout(timeout)
      signal?.removeEventListener('abort', abort)
      onStateChange?.(state)
      resolve()
    }

    function abort() {
      try { source.stop() } catch {}
      finish('idle')
    }

    const timeout = setTimeout(() => {
      if (settled) return
      try { source.stop() } catch {}
      settled = true
      signal?.removeEventListener('abort', abort)
      onStateChange?.('unavailable')
      reject(new Error('TTS 音频播放超时'))
    }, timeoutMs)

    if (signal?.aborted) {
      abort()
      return
    }

    signal?.addEventListener('abort', abort, { once: true })
    source.onended = () => finish('idle')
    source.start()
    onStateChange?.('playing')
  })
}
