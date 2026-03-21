import { useState, useEffect } from 'react'

/**
 * Loads config on mount. If intervalMs is provided, also polls on that interval.
 * Returns { config, reload } where reload() manually triggers a fresh load.
 */
export function useConfig(intervalMs = null) {
  const [config, setConfig] = useState(null)

  useEffect(() => {
    const invoke = window.__TAURI_INTERNALS__.invoke
    const load = () => invoke('get_config').then(setConfig)

    load()
    if (!intervalMs) return
    const id = setInterval(load, intervalMs)
    return () => clearInterval(id)
  }, [intervalMs])

  const reload = () => {
    window.__TAURI_INTERNALS__.invoke('get_config').then(setConfig)
  }

  return { config, reload }
}