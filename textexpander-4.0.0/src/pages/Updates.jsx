import { useEffect, useState } from 'react'
import { RefreshCw, CheckCircle, AlertCircle, Download, ExternalLink } from 'lucide-react'

const CURRENT_VERSION = '4.0.0'
const GITHUB_REPO = 'klazorix/textexpander'

function newerVersion(latest, current) {
  const a = latest.replace(/^v/, '').split('.').map(Number)
  const b = current.replace(/^v/, '').split('.').map(Number)
  for (let i = 0; i < 3; i++) {
    if ((a[i] ?? 0) > (b[i] ?? 0)) return true
    if ((a[i] ?? 0) < (b[i] ?? 0)) return false
  }
  return false
}

function formatDate(iso) {
  const d = new Date(iso)
  return d.toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric' })
}

function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export default function Updates() {
  const [status, setStatus] = useState('idle')
  const [release, setRelease] = useState(null)
  const [checkedAt, setCheckedAt] = useState(null)

  const check = async () => {
    setStatus('checking')
    try {
      const res = await fetch(`https://api.github.com/repos/${GITHUB_REPO}/releases/latest`)
      if (!res.ok) throw new Error('GitHub API error')
      const data = await res.json()
      setRelease(data)
      setCheckedAt(new Date())
      setStatus(newerVersion(data.tag_name, CURRENT_VERSION) ? 'available' : 'uptodate')
    } catch {
      setStatus('error')
    }
  }

  useEffect(() => { check() }, [])

  const assets = release?.assets ?? []

  return (
    <div className="max-w-2xl mx-auto">

      <div className="mb-10">
        <h1 className="text-3xl font-bold text-white">Updates</h1>
        <p className="text-gray-400 mt-1">Text Expander v{CURRENT_VERSION}</p>
      </div>

      <div className={`rounded-2xl border p-6 mb-6 transition-colors ${
        status === 'available'
          ? 'bg-orange-500/5 border-orange-500/30'
          : status === 'uptodate'
          ? 'bg-emerald-500/5 border-emerald-500/30'
          : status === 'error'
          ? 'bg-red-500/5 border-red-500/30'
          : 'bg-gray-900 border-gray-800'
      }`}>
        <div className="flex items-center justify-between gap-4">
          <div className="flex items-center gap-4">
            {status === 'checking' && (
              <RefreshCw size={28} className="text-blue-400 animate-spin shrink-0" />
            )}
            {status === 'uptodate' && (
              <CheckCircle size={28} className="text-emerald-400 shrink-0" />
            )}
            {status === 'available' && (
              <AlertCircle size={28} className="text-orange-400 shrink-0" />
            )}
            {status === 'error' && (
              <AlertCircle size={28} className="text-red-400 shrink-0" />
            )}
            {status === 'idle' && (
              <RefreshCw size={28} className="text-gray-600 shrink-0" />
            )}

            <div>
              {status === 'checking' && (
                <>
                  <p className="text-white font-semibold">Checking for updates...</p>
                  <p className="text-gray-400 text-sm mt-0.5">Contacting GitHub</p>
                </>
              )}
              {status === 'uptodate' && (
                <>
                  <p className="text-white font-semibold">You're up to date</p>
                  <p className="text-gray-400 text-sm mt-0.5">
                    Text Expander {CURRENT_VERSION} is the latest version
                  </p>
                </>
              )}
              {status === 'available' && (
                <>
                  <p className="text-white font-semibold">
                    Update available - {release.tag_name}
                  </p>
                  <p className="text-gray-400 text-sm mt-0.5">
                    Released {formatDate(release.published_at)}
                  </p>
                </>
              )}
              {status === 'error' && (
                <>
                  <p className="text-white font-semibold">Couldn't check for updates</p>
                  <p className="text-gray-400 text-sm mt-0.5">Check your internet connection</p>
                </>
              )}
            </div>
          </div>

          <button
            onClick={check}
            disabled={status === 'checking'}
            className="flex items-center gap-2 px-4 py-2 rounded-xl bg-gray-800 hover:bg-gray-700 text-white text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed shrink-0"
          >
            <RefreshCw size={14} className={status === 'checking' ? 'animate-spin' : ''} />
            Check
          </button>
        </div>

        {checkedAt && status !== 'checking' && (
          <p className="text-xs text-gray-600 mt-4">
            Last checked {checkedAt.toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}
          </p>
        )}
      </div>

      {release && status === 'available' && (
        <>
          {release.body && (
            <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 mb-4">
              <h2 className="text-sm font-semibold text-gray-400 uppercase tracking-widest mb-4">
                What's New
              </h2>
              <div className="text-gray-300 text-sm leading-relaxed whitespace-pre-wrap">
                {release.body}
              </div>
            </div>
          )}

          {assets.length > 0 && (
            <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 mb-4">
              <h2 className="text-sm font-semibold text-gray-400 uppercase tracking-widest mb-4">
                Downloads
              </h2>
              <div className="flex flex-col gap-2">
                {assets.map((asset, i) => (
                  
                    key={i}
                    href={asset.browser_download_url}
                    target="_blank"
                    rel="noreferrer"
                    className="flex items-center justify-between px-4 py-3 bg-gray-800 hover:bg-gray-700 rounded-xl transition-colors group"
                  >
                    <div className="flex items-center gap-3 min-w-0">
                      <Download size={15} className="text-gray-400 shrink-0" />
                      <span className="text-white text-sm truncate">{asset.name}</span>
                    </div>
                    <div className="flex items-center gap-3 shrink-0">
                      <span className="text-gray-500 text-xs">{formatBytes(asset.size)}</span>
                      <ExternalLink size={13} className="text-gray-600 group-hover:text-gray-400 transition-colors" />
                    </div>
                  </a>
                ))}
              </div>
            </div>
          )}

          
            href={release.html_url}
            target="_blank"
            rel="noreferrer"
            className="flex items-center justify-center gap-2 w-full py-3 rounded-2xl bg-gray-900 border border-gray-800 hover:border-gray-700 text-gray-400 hover:text-white text-sm transition-colors"
          >
            <ExternalLink size={14} />
            View on GitHub
          </a>
        </>
      )}
    </div>
  )
}