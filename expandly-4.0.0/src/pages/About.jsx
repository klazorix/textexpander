import { useEffect, useState } from 'react'
import { Github, BookOpen, ExternalLink, X, Heart } from 'lucide-react'
import logo from '../../src-tauri/icons/128x128.png'

function ContributorCard({ name, role, open }) {
  const [avatar, setAvatar] = useState(null)

  useEffect(() => {
    fetch(`https://api.github.com/users/${name}`)
      .then(r => r.json())
      .then(data => setAvatar(data.avatar_url))
      .catch(() => { })
  }, [name])

  return (
    <div className="bg-gray-900 border border-gray-800 rounded-xl px-5 py-4 flex items-center justify-between hover:border-gray-700 transition-colors">
      <div className="flex items-center gap-3">
        {avatar ? (
          <img src={avatar} alt={name} className="w-8 h-8 rounded-full" />
        ) : (
          <div className="w-8 h-8 rounded-full bg-gray-800" />
        )}
        <div>
          <p className="text-white text-sm font-medium">@{name}</p>
          <p className="text-gray-500 text-xs mt-0.5">{role}</p>
        </div>
      </div>
      <button
        onClick={() => open(`https://github.com/${name}`)}
        className="flex items-center gap-1.5 text-gray-500 hover:text-blue-400 text-xs transition-colors"
      >
        <Github size={13} />
        @{name}
      </button>
    </div>
  )
}

export default function About() {
  const [appVersion, setAppVersion] = useState('...')
  const [showLicense, setShowLicense] = useState(false)

  useEffect(() => {
    const { invoke } = window.__TAURI_INTERNALS__
    invoke('get_app_version').then(setAppVersion)
  }, [])

  const majorVersion = appVersion.split('.')[0]

  const open = (url) => {
    const { invoke } = window.__TAURI_INTERNALS__
    invoke('open_url', { url }).catch(() => {
      window.open(url, '_blank')
    })
  }

  return (
    <div className="max-w-2xl mx-auto">

      <div className="flex flex-col items-center text-center py-10 mb-8">
        <img
          src={logo}
          alt="Expandly"
          className="w-20 h-20 rounded-2xl mb-5 shadow-lg"
        />
        <h1 className="text-3xl font-bold text-white mb-1">
          Expandly {majorVersion}
        </h1>
        <p className="text-gray-500 text-sm mb-5">v{appVersion}</p>
        <p className="text-gray-400 text-sm leading-relaxed max-w-md">
          Expandly is a fast, lightweight text expander for desktop.
          Type a short trigger or press a hotkey and it instantly replaces it with your saved snippet, so you can keep working without taking your hands off the keyboard.
        </p>
      </div>

      <div className="grid grid-cols-2 gap-3 mb-8">
        <button
          onClick={() => open('https://github.com/klazorix/expandly')}
          className="flex items-center justify-center gap-2 bg-gray-900 border border-gray-800 hover:border-gray-700 rounded-xl py-3 text-gray-300 hover:text-white text-sm transition-colors"
        >
          <Github size={16} />
          GitHub
        </button>
        <button
          onClick={() => open('https://github.com/klazorix/expandly/wiki')}
          className="flex items-center justify-center gap-2 bg-gray-900 border border-gray-800 hover:border-gray-700 rounded-xl py-3 text-gray-300 hover:text-white text-sm transition-colors"
        >
          <BookOpen size={16} />
          Wiki
        </button>
      </div>

      <div className="mb-8">
        <p className="text-xs font-semibold text-gray-500 uppercase tracking-widest mb-3">
          Contributors
        </p>
        <div className="flex flex-col gap-2">
          <div className="flex flex-col gap-2">
            {[
              { name: 'klazorix', role: 'Lead Developer & Maintainer' },
              { name: 'encryptednoobi', role: 'Logo Designer' },
            ].map(c => (
              <ContributorCard key={c.name} name={c.name} role={c.role} open={open} />
            ))}
          </div>
        </div>
      </div>

      <div className="mb-8">
        <p className="text-xs font-semibold text-gray-500 uppercase tracking-widest mb-3">
          Acknowledgements
        </p>
        <div className="bg-gray-900 border border-gray-800 rounded-xl px-5 py-4">
          <p className="text-gray-400 text-sm leading-relaxed">
            Expandly is built on great open source work.
            Special thanks to the teams behind{' '}
            <span className="text-gray-300">Tauri</span>,{' '}
            <span className="text-gray-300">React</span>,{' '}
            <span className="text-gray-300">Rust</span>,{' '}
            <span className="text-gray-300">rdev</span>,{' '}
            <span className="text-gray-300">enigo</span>, and{' '}
            <span className="text-gray-300">rodio</span>{' '}
            who made this project possible.
          </p>
        </div>
      </div>

      <div className="mb-8">
        <p className="text-xs font-semibold text-gray-500 uppercase tracking-widest mb-3">
          Legal
        </p>
        <button
          onClick={() => setShowLicense(true)}
          className="w-full flex items-center justify-between bg-gray-900 border border-gray-800 hover:border-gray-700 rounded-xl px-5 py-4 text-left transition-colors group"
        >
          <div>
            <p className="text-white text-sm font-medium">License</p>
            <p className="text-gray-500 text-xs mt-0.5">View the full license for Expandly {majorVersion}</p>
          </div>
          <ExternalLink size={15} className="text-gray-600 group-hover:text-gray-400 transition-colors shrink-0" />
        </button>
      </div>

      <div className="flex items-center justify-center gap-1.5 pb-8">
        <Heart size={12} className="text-gray-700" />
        <p className="text-gray-700 text-xs">Made with care by klazorix</p>
      </div>

      {showLicense && (
        <LicenseModal
          majorVersion={majorVersion}
          onClose={() => setShowLicense(false)}
        />
      )}
    </div>
  )
}

function LicenseModal({ majorVersion, onClose }) {
  const [license, setLicense] = useState(null)
  const [error, setError] = useState(false)

  useEffect(() => {
    fetch('https://raw.githubusercontent.com/klazorix/expandly/expandly-4/LICENSE')
      .then(r => {
        if (!r.ok) throw new Error()
        return r.text()
      })
      .then(setLicense)
      .catch(() => setError(true))
  }, [])

  return (
    <div className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 p-6">
      <div className="bg-gray-900 border border-gray-700 rounded-2xl w-full max-w-2xl max-h-[80vh] flex flex-col shadow-2xl">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-800 shrink-0">
          <h2 className="text-white font-semibold">License — Expandly {majorVersion}</h2>
          <button
            onClick={onClose}
            className="text-gray-500 hover:text-white transition-colors"
          >
            <X size={20} />
          </button>
        </div>
        <div className="overflow-y-auto px-6 py-4 flex-1">
          {!license && !error && (
            <p className="text-gray-500 text-sm">Loading license...</p>
          )}
          {error && (
            <div className="text-red-400 text-sm">
              <p className="mb-2">Could not load license. Please refer to the expandly-{majorVersion} branch on our repo.</p>
              <button
                onClick={() => {
                  const { invoke } = window.__TAURI_INTERNALS__
                  invoke('open_url', { url: `https://github.com/klazorix/expandly/blob/expandly-${majorVersion}/LICENSE` })
                }}
                className="underline hover:text-red-300 transition-colors"
              >
                View LICENSE on GitHub
              </button>
            </div>
          )}
          {license && (
            <pre className="text-gray-300 text-xs leading-relaxed whitespace-pre-wrap font-mono">
              {license}
            </pre>
          )}
        </div>
      </div>
    </div>
  )
}