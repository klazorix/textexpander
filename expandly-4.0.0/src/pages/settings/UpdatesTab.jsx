import { useEffect, useState } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { RefreshCw, CheckCircle, AlertCircle, Download, ExternalLink, X } from 'lucide-react'
import { useInvoke } from '../../hooks/useInvoke'
import { formatBytes, formatDate, newerVersion } from './utils'

function ChangelogModal({ release, onClose }) {
  const invoke = useInvoke()
  const openUrl = url => invoke('open_url', { url }).catch(() => { })

  return (
    <div className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 p-6">
      <div className="bg-gray-900 border border-gray-700 rounded-2xl w-full max-w-2xl max-h-[80vh] flex flex-col shadow-2xl">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-800 shrink-0">
          <div>
            <h2 className="text-white font-semibold">{release.tag_name}</h2>
            <p className="text-gray-500 text-xs mt-0.5">
              Released {formatDate(release.published_at)}
              {release.prerelease && (
                <span className="ml-2 bg-orange-500/20 text-orange-300 px-2 py-0.5 rounded-full">Pre-release</span>
              )}
            </p>
          </div>
          <button onClick={onClose} className="text-gray-500 hover:text-white transition-colors">
            <X size={20} />
          </button>
        </div>

        <div className="overflow-y-auto px-6 py-4 flex-1">
          {release.body ? (
            <ReactMarkdown
              remarkPlugins={[remarkGfm]}
              components={{
                h1: ({ children }) => <p className="text-white font-bold text-base mb-2 mt-4">{children}</p>,
                h2: ({ children }) => <p className="text-white font-bold text-sm mb-2 mt-4">{children}</p>,
                h3: ({ children }) => <p className="text-white font-semibold text-sm mb-1 mt-3">{children}</p>,
                p: ({ children }) => <p className="text-gray-300 text-sm leading-relaxed mb-3">{children}</p>,
                li: ({ children }) => <li className="text-gray-300 text-sm leading-relaxed ml-4 list-disc mb-0.5">{children}</li>,
                ul: ({ children }) => <ul className="mb-3 space-y-0.5">{children}</ul>,
                ol: ({ children }) => <ol className="mb-3 space-y-0.5 list-decimal ml-4">{children}</ol>,
                strong: ({ children }) => <strong className="text-white font-semibold">{children}</strong>,
                code: ({ children }) => <code className="text-blue-300 bg-blue-500/10 px-1 rounded text-xs">{children}</code>,
                hr: () => <hr className="border-gray-700 my-4" />,
                a: ({ href, children }) => (
                  <button onClick={() => openUrl(href)} className="text-blue-400 hover:underline">{children}</button>
                ),
                table: ({ children }) => <table className="w-full text-sm text-gray-300 border-collapse mb-3">{children}</table>,
                th: ({ children }) => <th className="text-left text-white font-semibold border-b border-gray-700 pb-1 pr-4">{children}</th>,
                td: ({ children }) => <td className="border-b border-gray-800 py-1 pr-4">{children}</td>,
              }}
            >
              {release.body}
            </ReactMarkdown>
          ) : (
            <p className="text-gray-500 text-sm">No release notes provided.</p>
          )}
        </div>

        {release.assets?.length > 0 && (
          <div className="px-6 py-4 border-t border-gray-800 shrink-0">
            <p className="text-xs font-semibold text-gray-500 uppercase tracking-widest mb-3">Downloads</p>
            <div className="flex flex-col gap-2">
              {release.assets.map((asset, i) => (
                <div key={i} className="flex items-center justify-between px-4 py-3 bg-gray-800 rounded-xl">
                  <div className="flex items-center gap-3 min-w-0">
                    <Download size={15} className="text-gray-400 shrink-0" />
                    <span className="text-white text-sm truncate">{asset.name}</span>
                  </div>
                  <div className="flex items-center gap-3 shrink-0">
                    <span className="text-gray-500 text-xs">{formatBytes(asset.size)}</span>
                    <button onClick={() => openUrl(asset.browser_download_url)} className="text-blue-400 hover:text-blue-300 transition-colors">
                      <ExternalLink size={13} />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        <div className="px-6 py-4 border-t border-gray-800 shrink-0">
          <button
            onClick={() => openUrl(release.html_url)}
            className="flex items-center justify-center gap-2 w-full py-2.5 rounded-xl bg-gray-800 hover:bg-gray-700 text-gray-400 hover:text-white text-sm transition-colors"
          >
            <ExternalLink size={14} />
            View on GitHub
          </button>
        </div>
      </div>
    </div>
  )
}

export default function UpdatesTab() {
  const invoke = useInvoke()
  const [status, setStatus] = useState('idle')
  const [latestRelease, setLatestRelease] = useState(null)
  const [currentRelease, setCurrentRelease] = useState(null)
  const [checkedAt, setCheckedAt] = useState(null)
  const [appVersion, setAppVersion] = useState('')
  const [showChangelog, setShowChangelog] = useState(null)

  const checkForUpdates = async version => {
    setStatus('checking')
    try {
      const allRes = await fetch('https://api.github.com/repos/klazorix/Expandly/releases')
      if (!allRes.ok) throw new Error()
      const allReleases = await allRes.json()
      setCurrentRelease(allReleases.find(r => r.tag_name === version || r.tag_name === `v${version}`) ?? null)

      const latestRes = await fetch('https://api.github.com/repos/klazorix/Expandly/releases/latest')
      if (!latestRes.ok) throw new Error()
      const candidate = await latestRes.json()
      if (!candidate) throw new Error()

      setLatestRelease(candidate)
      setCheckedAt(new Date())
      setStatus(newerVersion(candidate.tag_name, version) ? 'available' : 'uptodate')
    } catch {
      setStatus('error')
    }
  }

  useEffect(() => {
    invoke('get_app_version').then(version => {
      setAppVersion(version)
      checkForUpdates(version)
    })
  }, [])

  return (
    <div>
      <div className={`rounded-2xl border p-6 mb-4 transition-colors ${
        status === 'available' ? 'bg-orange-500/5 border-orange-500/30'
          : status === 'uptodate' ? 'bg-emerald-500/5 border-emerald-500/30'
            : status === 'error' ? 'bg-red-500/5 border-red-500/30'
              : 'bg-gray-900 border-gray-800'
      }`}>
        <div className="flex items-center justify-between gap-4">
          <div className="flex items-center gap-4">
            {status === 'checking' && <RefreshCw size={24} className="text-blue-400 animate-spin shrink-0" />}
            {status === 'uptodate' && <CheckCircle size={24} className="text-emerald-400 shrink-0" />}
            {status === 'available' && <AlertCircle size={24} className="text-orange-400 shrink-0" />}
            {status === 'error' && <AlertCircle size={24} className="text-red-400 shrink-0" />}
            {status === 'idle' && <RefreshCw size={24} className="text-gray-600 shrink-0" />}
            <div>
              {status === 'checking' && <><p className="text-white font-semibold">Checking for updates...</p><p className="text-gray-400 text-sm mt-0.5">Contacting GitHub</p></>}
              {status === 'uptodate' && <><p className="text-white font-semibold">You are up to date</p><p className="text-gray-400 text-sm mt-0.5">Expandly {appVersion} is the latest version</p></>}
              {status === 'available' && latestRelease && (
                <>
                  <p className="text-white font-semibold">Update available â€” {latestRelease.tag_name}</p>
                  <p className="text-gray-400 text-sm mt-0.5">
                    Released {formatDate(latestRelease.published_at)}
                    {latestRelease.prerelease && <span className="ml-2 text-xs bg-orange-500/20 text-orange-300 px-2 py-0.5 rounded-full">Pre-release</span>}
                  </p>
                </>
              )}
              {status === 'error' && <><p className="text-white font-semibold">Could not check for updates</p><p className="text-gray-400 text-sm mt-0.5">Check your internet connection</p></>}
            </div>
          </div>
          <div className="flex items-center gap-2 shrink-0">
            {status === 'available' && latestRelease && (
              <button
                onClick={() => setShowChangelog(latestRelease)}
                className="flex items-center gap-2 px-4 py-2 rounded-xl bg-orange-500/10 hover:bg-orange-500/20 text-orange-300 text-sm font-medium transition-colors"
              >
                What's New
              </button>
            )}
            <button
              onClick={() => checkForUpdates(appVersion)}
              disabled={status === 'checking'}
              className="flex items-center gap-2 px-4 py-2 rounded-xl bg-gray-800 hover:bg-gray-700 text-white text-sm font-medium transition-colors disabled:opacity-40"
            >
              <RefreshCw size={14} className={status === 'checking' ? 'animate-spin' : ''} />
              Check
            </button>
          </div>
        </div>
        {checkedAt && status !== 'checking' && (
          <p className="text-xs text-gray-600 mt-4">
            Last checked {checkedAt.toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })}
          </p>
        )}
      </div>

      {currentRelease && (
        <button
          onClick={() => setShowChangelog(currentRelease)}
          className="w-full flex items-center justify-between bg-gray-900 border border-gray-800 hover:border-gray-700 rounded-xl px-5 py-4 text-left transition-colors group"
        >
          <div>
            <p className="text-white text-sm font-medium">Release Notes</p>
            <p className="text-gray-500 text-xs mt-0.5">View changelog for your current version {appVersion}</p>
          </div>
          <ExternalLink size={15} className="text-gray-600 group-hover:text-gray-400 transition-colors shrink-0" />
        </button>
      )}

      {showChangelog && <ChangelogModal release={showChangelog} onClose={() => setShowChangelog(null)} />}
    </div>
  )
}
