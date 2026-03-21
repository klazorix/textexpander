import { useState, useEffect } from 'react'
import { useInvoke } from './useInvoke'

/**
 * Loads config on mount. If intervalMs is provided, also polls on that interval.
 * Returns { config, reload } where reload() manually triggers a fresh load.
 */
export function useConfig(intervalMs = null) {
  const invoke = useInvoke()
  const [config, setConfig] = useState(null)

  const load = () => invoke('get_config').then(setConfig)

  useEffect(() => {
    load()
    if (!intervalMs) return
    const id = setInterval(load, intervalMs)
    return () => clearInterval(id)
  }, [])

  return { config, reload: load }
}
