/**
 * Generic async data fetching composable with loading/error states
 */

import { ref, Ref } from 'vue'
import type { AsyncState, LoadingState } from '@/shared/types'

export interface UseAsyncOptions<T> {
  immediate?: boolean
  onSuccess?: (data: T) => void
  onError?: (error: Error) => void
  initialData?: T
}

export function useAsync<T>(
  asyncFn: () => Promise<T>,
  options: UseAsyncOptions<T> = {}
) {
  const {
    immediate = false,
    onSuccess,
    onError,
    initialData = null
  } = options

  const data: Ref<T | null> = ref(initialData)
  const loading = ref(false)
  const error: Ref<string | null> = ref(null)
  const state: Ref<LoadingState> = ref('idle')

  async function execute(...args: any[]): Promise<T | null> {
    loading.value = true
    error.value = null
    state.value = 'loading'

    try {
      const result = await asyncFn(...args)
      data.value = result
      state.value = 'success'
      
      if (onSuccess) {
        onSuccess(result)
      }
      
      return result
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'An error occurred'
      error.value = errorMessage
      state.value = 'error'
      
      if (onError) {
        onError(err as Error)
      }
      
      return null
    } finally {
      loading.value = false
    }
  }

  function reset() {
    data.value = initialData
    loading.value = false
    error.value = null
    state.value = 'idle'
  }

  if (immediate) {
    execute()
  }

  return {
    data,
    loading,
    error,
    state,
    execute,
    reset
  }
}
