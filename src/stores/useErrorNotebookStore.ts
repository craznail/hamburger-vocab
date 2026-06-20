import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import * as errorApi from '../api/errorItem'
import * as authApi from '../api/auth'

const STALE_MS = 8_000

export const useErrorNotebookStore = defineStore('errorNotebook', () => {
  const notebooks = ref<errorApi.ErrorNotebook[]>([])
  const items = ref<errorApi.ErrorItem[]>([])
  const auth = ref<authApi.AuthStatus>({ loggedIn: false })
  const initialized = ref(false)
  const loading = ref(false)
  const syncError = ref('')
  const lastLoadedAt = ref(0)
  let inflightLoad: Promise<void> | null = null
  let inflightSync: Promise<void> | null = null

  const dueCount = computed(() => notebooks.value.reduce((sum, notebook) => sum + (notebook.dueCount || 0), 0))
  const pendingCount = computed(() => items.value.filter(item => item.syncStatus !== 'synced').length)
  const hasData = computed(() => notebooks.value.length > 0 || items.value.length > 0 || initialized.value)

  async function syncRemote(): Promise<void> {
    if (inflightSync) {
      return inflightSync
    }

    const request = (async () => {
      try {
        await errorApi.syncErrorItems()
        syncError.value = ''
      } catch (e) {
        syncError.value = e instanceof Error ? e.message : String(e)
        // A failed sync often means the credentials are gone (expired/revoked).
        // Re-read the auth status so the UI drops back to the login screen.
        try {
          auth.value = await authApi.getAuthStatus()
        } catch {
          /* keep the cached status if the status call itself fails */
        }
        throw e
      } finally {
        inflightSync = null
      }
    })()

    inflightSync = request
    return request
  }

  async function refresh(force = false, pullRemote = true): Promise<void> {
    if (inflightLoad && !force) {
      return inflightLoad
    }

    if (!force && lastLoadedAt.value && Date.now() - lastLoadedAt.value < STALE_MS) {
      initialized.value = true
      return
    }

    loading.value = true
    const request = (async () => {
      try {
        const nextAuth = await authApi.getAuthStatus()
        auth.value = nextAuth

        if (pullRemote && nextAuth.loggedIn) {
          try {
            await syncRemote()
          } catch {
            // Keep showing the local cache even if the remote endpoint is temporarily unavailable.
          }
        }

        const [nextNotebooks, nextItems] = await Promise.all([
          errorApi.getErrorNotebooks(),
          errorApi.getErrorItems(),
        ])
        notebooks.value = nextNotebooks
        items.value = nextItems
        lastLoadedAt.value = Date.now()
        initialized.value = true
      } finally {
        loading.value = false
        inflightLoad = null
      }
    })()

    inflightLoad = request
    return request
  }

  async function ensureFresh(): Promise<void> {
    await refresh(!initialized.value)
  }

  function invalidate(): void {
    lastLoadedAt.value = 0
  }

  function prime(): void {
    if (initialized.value || inflightLoad) return
    void refresh(true)
  }

  return {
    notebooks,
    items,
    auth,
    initialized,
    loading,
    syncError,
    dueCount,
    pendingCount,
    hasData,
    refresh,
    ensureFresh,
    invalidate,
    prime,
    syncRemote,
  }
})
