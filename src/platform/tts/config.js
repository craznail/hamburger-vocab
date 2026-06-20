const STORAGE_KEY = 'recall.tts.settings.v1'

export const TTS_PROVIDERS = {
  azure: {
    id: 'azure',
    label: 'Azure Speech',
    defaultVoice: 'en-US-JennyNeural',
    defaultLanguage: 'en-US'
  },
  aliyun: {
    id: 'aliyun',
    label: '阿里云 DashScope',
    defaultVoice: 'Jennifer',
    defaultLanguage: 'Auto'
  }
}

function envDefaults() {
  const azureKey = import.meta.env.VITE_AZURE_SPEECH_KEY || ''
  const aliyunKey = import.meta.env.VITE_DASHSCOPE_API_KEY || ''
  const provider = import.meta.env.VITE_TTS_PROVIDER || 'azure'

  return {
    provider,
    azure: {
      apiKey: azureKey,
      region: import.meta.env.VITE_AZURE_SPEECH_REGION || 'eastus',
      voice: import.meta.env.VITE_AZURE_SPEECH_VOICE || 'en-US-JennyNeural',
      language: import.meta.env.VITE_AZURE_SPEECH_LANGUAGE || 'en-US',
      rate: import.meta.env.VITE_AZURE_SPEECH_RATE || '0%',
      volume: import.meta.env.VITE_AZURE_SPEECH_VOLUME || '+35%'
    },
    aliyun: {
      apiKey: aliyunKey,
      model: import.meta.env.VITE_DASHSCOPE_TTS_MODEL || 'qwen3-tts-flash',
      voice: import.meta.env.VITE_DASHSCOPE_TTS_VOICE || 'Jennifer',
      language: 'Auto',
      rate: '0%'
    }
  }
}

function mergeSettings(base, saved = {}) {
  return {
    ...base,
    ...saved,
    azure: { ...base.azure, ...saved.azure },
    aliyun: { ...base.aliyun, ...saved.aliyun }
  }
}

export function getTtsSettings() {
  const defaults = envDefaults()
  try {
    const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) || '{}')
    return mergeSettings(defaults, saved)
  } catch {
    return defaults
  }
}

export function saveTtsSettings(settings) {
  const normalized = mergeSettings(envDefaults(), settings)
  localStorage.setItem(STORAGE_KEY, JSON.stringify(normalized))
  return normalized
}

export function activeProviderSettings(settings = getTtsSettings()) {
  const provider = TTS_PROVIDERS[settings.provider] ? settings.provider : 'azure'
  return {
    provider,
    ...settings[provider]
  }
}

export function makeTtsCacheKey(text, settings = getTtsSettings()) {
  const active = activeProviderSettings(settings)
  return [
    'v3',
    active.provider,
    active.region || '',
    active.model || '',
    active.voice || '',
    active.language || '',
    active.rate || '',
    active.volume || '',
    text.toLowerCase().trim()
  ].join(':')
}
