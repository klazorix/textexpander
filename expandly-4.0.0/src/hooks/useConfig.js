import { useState, useEffect } from 'react'

export function useConfig(intervalMs = null) {
  const [config, setConfig] = useState(null)

  useEffect(() => {
    const load = () => window.__TAURI_INTERNALS__.invoke('get_config').then(setConfig)

    load()
    if (intervalMs == null) return

    const id = setInterval(load, intervalMs)
    return () => clearInterval(id)
  }, [intervalMs])

  return {
    config,
    reload: () => window.__TAURI_INTERNALS__.invoke('get_config').then(setConfig),
  }
}
