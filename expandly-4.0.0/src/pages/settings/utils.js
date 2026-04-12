export function newerVersion(latest, current) {
  const clean = value => value.replace(/^v/, '')
  const parse = value => {
    const normalized = clean(value)
    const prerelease = normalized.match(/^(\d+\.\d+\.\d+)b(\d+)$/)

    if (prerelease) {
      return {
        parts: prerelease[1].split('.').map(Number),
        pre: Number(prerelease[2]),
      }
    }

    return { parts: normalized.split('.').map(Number), pre: null }
  }

  const candidate = parse(latest)
  const installed = parse(current)

  for (let i = 0; i < 3; i++) {
    if ((candidate.parts[i] ?? 0) > (installed.parts[i] ?? 0)) return true
    if ((candidate.parts[i] ?? 0) < (installed.parts[i] ?? 0)) return false
  }

  if (candidate.pre === null && installed.pre !== null) return true
  if (candidate.pre !== null && installed.pre === null) return false
  if (candidate.pre !== null && installed.pre !== null) return candidate.pre > installed.pre
  return false
}

export function formatDate(iso) {
  return new Date(iso).toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric' })
}

export function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
