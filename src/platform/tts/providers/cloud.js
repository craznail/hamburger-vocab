import { invoke } from '@tauri-apps/api/core'
import { activeProviderSettings } from '../config'

function decodeBase64(value) {
  const binary = atob(value)
  const bytes = new Uint8Array(binary.length)
  for (let index = 0; index < binary.length; index++) {
    bytes[index] = binary.charCodeAt(index)
  }
  return bytes.buffer
}

export async function synthesizeCloudSpeech(text, settings) {
  const active = activeProviderSettings(settings)
  if (!active.apiKey) {
    throw new Error(`${active.provider === 'azure' ? 'Azure' : '阿里云'} API Key 未配置`)
  }

  const response = await invoke('synthesize_speech', {
    request: {
      provider: active.provider,
      text,
      apiKey: active.apiKey,
      region: active.region || null,
      voice: active.voice || null,
      language: active.language || null,
      rate: active.rate || null,
      volume: active.volume || null,
      model: active.model || null
    }
  })

  return {
    buffer: decodeBase64(response.audioBase64),
    contentType: response.contentType || 'audio/mpeg',
    provider: response.provider,
    voice: response.voice
  }
}
