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
  const lastLoadedAt = ref(0)
  let inflightLoad: Promise<void> | null = null

  const dueCount = computed(() => notebooks.value.reduce((sum, notebook) => sum + (notebook.dueCount || 0), 0))
  const pendingCount = computed(() => items.value.filter(item => item.syncStatus !== 'synced').length)
  const hasData = computed(() => notebooks.value.length > 0 || items.value.length > 0 || initialized.value)

  async function refresh(force = false): Promise<void> {
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
        const [nextAuth, nextNotebooks, nextItems] = await Promise.all([
          authApi.getAuthStatus(),
          errorApi.getErrorNotebooks(),
          errorApi.getErrorItems(),
        ])
        auth.value = nextAuth
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
    dueCount,
    pendingCount,
    hasData,
    refresh,
    ensureFresh,
    invalidate,
    prime,
  }
})
