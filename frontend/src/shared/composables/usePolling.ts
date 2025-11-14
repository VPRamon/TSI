/**
 * Composable for polling a data source at regular intervals
 */

import { ref, onMounted, onUnmounted } from 'vue'

export interface UsePollingOptions {
  interval?: number
  immediate?: boolean
  enabled?: boolean
}

export function usePolling(
  callback: () => void | Promise<void>,
  options: UsePollingOptions = {}
) {
  const {
    interval = 2000,
    immediate = true,
    enabled = true
  } = options

  const isPolling = ref(false)
  let timerId: number | null = null

  async function execute() {
    if (!enabled) return
    
    try {
      await callback()
    } catch (error) {
      console.error('Polling error:', error)
    }
  }

  function start() {
    if (isPolling.value || !enabled) return
    
    isPolling.value = true
    
    if (immediate) {
      execute()
    }
    
    timerId = window.setInterval(execute, interval)
  }

  function stop() {
    isPolling.value = false
    
    if (timerId !== null) {
      clearInterval(timerId)
      timerId = null
    }
  }

  function restart() {
    stop()
    start()
  }

  onMounted(() => {
    if (enabled) {
      start()
    }
  })

  onUnmounted(() => {
    stop()
  })

  return {
    isPolling,
    start,
    stop,
    restart
  }
}
