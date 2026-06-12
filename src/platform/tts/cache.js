const DATABASE_NAME = 'tts-audio-cache'
const STORE_NAME = 'audio'
let databasePromise = null

function openCache() {
  if (databasePromise) return databasePromise
  databasePromise = new Promise((resolve, reject) => {
    const request = indexedDB.open(DATABASE_NAME, 2)
    request.onupgradeneeded = () => {
      if (!request.result.objectStoreNames.contains(STORE_NAME)) {
        request.result.createObjectStore(STORE_NAME)
      }
    }
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => {
      databasePromise = null
      reject(request.error)
    }
  })
  return databasePromise
}

export async function getCachedAudio(key) {
  try {
    const database = await openCache()
    return await new Promise((resolve) => {
      const transaction = database.transaction(STORE_NAME, 'readonly')
      const request = transaction.objectStore(STORE_NAME).get(key)
      request.onsuccess = () => resolve(request.result || null)
      request.onerror = () => resolve(null)
    })
  } catch {
    return null
  }
}

export async function setCachedAudio(key, audio) {
  try {
    const database = await openCache()
    await new Promise((resolve) => {
      const transaction = database.transaction(STORE_NAME, 'readwrite')
      transaction.objectStore(STORE_NAME).put({
        buffer: audio.buffer,
        contentType: audio.contentType,
        timestamp: Date.now()
      }, key)
      transaction.oncomplete = resolve
      transaction.onerror = resolve
    })
  } catch {
    // Audio cache is an optimization; synthesis should still succeed without it.
  }
}
