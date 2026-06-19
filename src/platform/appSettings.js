const STORAGE_KEY = 'recall.app.settings.v1'

function defaults() {
  return {
    study: {
      enableMockDataFallback: false,
    },
    stats: {
      enableMockDataFallback: false,
    },
    errorNotebook: {
      enableMockDataFallback: false,
    },
  }
}

function mergeSettings(base, saved = {}) {
  return {
    ...base,
    ...saved,
    study: {
      ...base.study,
      ...saved.study,
    },
    stats: {
      ...base.stats,
      ...saved.stats,
    },
    errorNotebook: {
      ...base.errorNotebook,
      ...saved.errorNotebook,
    },
  }
}

export function getAppSettings() {
  const base = defaults()
  try {
    const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) || '{}')
    return mergeSettings(base, saved)
  } catch {
    return base
  }
}

export function saveAppSettings(settings) {
  const normalized = mergeSettings(defaults(), settings)
  localStorage.setItem(STORAGE_KEY, JSON.stringify(normalized))
  return normalized
}
