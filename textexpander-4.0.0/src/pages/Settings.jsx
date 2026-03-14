import { useState, useEffect, useRef } from 'react'
import { Settings as SettingsIcon, Palette, Database, RefreshCw, Upload, X, Volume2, CheckCircle, AlertCircle, Download, ExternalLink, Trash2 } from 'lucide-react'

const { invoke } = window.__TAURI_INTERNALS__

const CURRENT_VERSION = '4.0.0'
const GITHUB_REPO = 'klazorix/textexpander'

const tabs = [
  { id: 'engine', label: 'Engine', icon: SettingsIcon },
  { id: 'appearance', label: 'Appearance', icon: Palette },
  { id: 'data', label: 'Data', icon: Database },
  { id: 'updates', label: 'Updates', icon: RefreshCw },
]

const THEMES = [
  { id: 'starry-blue', label: 'Starry Blue', dark: true, swatch: '#161b27', accent: '#3b82f6' },
  { id: 'midnight', label: 'Midnight', dark: true, swatch: '#0a0a0a', accent: '#3b82f6' },
  { id: 'charcoal', label: 'Charcoal', dark: true, swatch: '#222222', accent: '#3b82f6' },
  { id: 'slate', label: 'Slate', dark: true, swatch: '#1e293b', accent: '#6366f1' },
  { id: 'deep-purple', label: 'Deep Purple', dark: true, swatch: '#16112b', accent: '#8b5cf6' },
  { id: 'forest', label: 'Forest', dark: true, swatch: '#111f1a', accent: '#10b981' },
  { id: 'crimson', label: 'Crimson', dark: true, swatch: '#1f1010', accent: '#ef4444' },
  { id: 'amber', label: 'Amber', dark: true, swatch: '#1f1810', accent: '#f59e0b' },
  { id: 'rose-pastel', label: 'Rose Pastel', dark: false, swatch: '#ffe4e9', accent: '#f43f5e' },
  { id: 'sky-pastel', label: 'Sky Pastel', dark: false, swatch: '#e0f2fe', accent: '#0ea5e9' },
  { id: 'mint-pastel', label: 'Mint Pastel', dark: false, swatch: '#dcfce7', accent: '#22c55e' },
  { id: 'lavender-pastel', label: 'Lavender Pastel', dark: false, swatch: '#f3e8ff', accent: '#a855f7' },
  { id: 'pearl', label: 'Pearl', dark: false, swatch: '#f0f0f0', accent: '#3b82f6' },
]

function Toggle({ value, onChange }) {
  return (
    <button
      onClick={() => onChange(!value)}
      className={`w-11 h-6 rounded-full transition-colors relative shrink-0 ${value ? 'bg-blue-600' : 'bg-gray-600'}`}
    >
      <span className={`absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-all ${value ? 'left-5' : 'left-0.5'}`} />
    </button>
  )
}

function SettingRow({ label, description, children }) {
  return (
    <div className="flex items-center justify-between gap-6 py-4 border-b border-gray-800 last:border-0">
      <div className="min-w-0">
        <p className="text-white text-sm font-medium">{label}</p>
        {description && <p className="text-gray-500 text-xs mt-0.5">{description}</p>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  )
}

function SectionLabel({ children }) {
  return (
    <p className="text-xs font-semibold text-gray-500 uppercase tracking-widest mb-3 mt-6 first:mt-0">
      {children}
    </p>
  )
}

function Card({ children }) {
  return (
    <div className="bg-gray-900 border border-gray-800 rounded-2xl px-6 mb-4">
      {children}
    </div>
  )
}

function EngineTab() {
  const [config, setConfig] = useState(null)
  const [enabled, setEnabled] = useState(false)
  const [soundEnabled, setSoundEnabled] = useState(false)
  const [soundPath, setSoundPath] = useState(null)
  const [soundName, setSoundName] = useState(null)
  const [launchAtStartup, setLaunchAtStartup] = useState(false)
  const [minimiseToTray, setMinimiseToTray] = useState(false)
  const [showInTaskbar, setShowInTaskbar] = useState(false)
  const fileRef = useRef()

  useEffect(() => {
    invoke('get_config').then(c => {
      setConfig(c)
      setEnabled(c.enabled)
      setSoundEnabled(c.sound_enabled)
      setSoundPath(c.sound_path ?? null)
      if (c.sound_path) setSoundName(c.sound_path.split(/[\\/]/).pop())
      setLaunchAtStartup(c.launch_at_startup ?? false)
      setMinimiseToTray(c.minimise_to_tray ?? false)
      setShowInTaskbar(c.show_in_taskbar ?? true)
    })
  }, [])

  const saveEngine = async (overrides = {}) => {
    await invoke('update_engine_settings', {
      enabled: overrides.enabled ?? enabled,
      soundEnabled: overrides.soundEnabled ?? soundEnabled,
      soundPath: overrides.soundPath !== undefined ? overrides.soundPath : soundPath,
    })
  }

  const saveSystem = async (overrides = {}) => {
    await invoke('update_system_settings', {
      launchAtStartup: overrides.launchAtStartup ?? launchAtStartup,
      minimiseToTray: overrides.minimiseToTray ?? minimiseToTray,
      showInTaskbar: overrides.showInTaskbar ?? showInTaskbar,
    })
  }

  const handleToggleEngine = (val) => { setEnabled(val); saveEngine({ enabled: val }) }
  const handleToggleSound = (val) => { setSoundEnabled(val); saveEngine({ soundEnabled: val }) }
  const handleStartup = (val) => { setLaunchAtStartup(val); saveSystem({ launchAtStartup: val }) }
  const handleTray = (val) => { setMinimiseToTray(val); saveSystem({ minimiseToTray: val }) }
  const handleTaskbar = (val) => { setShowInTaskbar(val); saveSystem({ showInTaskbar: val }) }

  const handleFileUpload = async (e) => {
    const file = e.target.files[0]
    if (!file) return
    const buffer = await file.arrayBuffer()
    const bytes = Array.from(new Uint8Array(buffer))
    const path = await invoke('save_sound_file', { fileName: file.name, fileData: bytes })
    setSoundPath(path)
    setSoundName(file.name)
    saveEngine({ soundPath: path })
  }

  const handleRemoveSound = () => {
    setSoundPath(null)
    setSoundName(null)
    saveEngine({ soundPath: null })
  }

  const previewSound = () => {
    if (!soundPath) return
    const audio = new Audio(`asset://localhost/${soundPath.replace(/\\/g, '/')}`)
    audio.play().catch(() => { })
  }

  return (
    <div>
      <SectionLabel>Engine</SectionLabel>
      <Card>
        <SettingRow
          label="Enable Engine"
          description="Master switch - when off, no triggers or hotkeys will fire"
        >
          <Toggle value={enabled} onChange={handleToggleEngine} />
        </SettingRow>
        {config && (
          <div className="py-3">
            <p className="text-xs text-gray-600">Expandly Engine {config.version}</p>
          </div>
        )}
      </Card>

      <SectionLabel>Expansion Sound</SectionLabel>
      <Card>
        <SettingRow
          label="Play Sound on Expansion"
          description="Plays a custom sound every time a snippet expands"
        >
          <Toggle value={soundEnabled} onChange={handleToggleSound} />
        </SettingRow>
        {soundEnabled && (
          <SettingRow
            label="Sound File"
            description="Upload a .mp3, .wav or .ogg file"
          >
            <div className="flex items-center gap-2">
              {soundName ? (
                <>
                  <button
                    onClick={previewSound}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-300 text-xs transition-colors"
                  >
                    <Volume2 size={13} />
                    {soundName}
                  </button>
                  <button
                    onClick={handleRemoveSound}
                    className="p-1.5 rounded-lg text-gray-500 hover:text-red-400 hover:bg-gray-800 transition-colors"
                  >
                    <X size={14} />
                  </button>
                </>
              ) : (
                <button
                  onClick={() => fileRef.current.click()}
                  className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-300 text-xs transition-colors"
                >
                  <Upload size={13} />
                  Upload Sound
                </button>
              )}
              <input
                ref={fileRef}
                type="file"
                accept=".mp3,.wav,.ogg"
                className="hidden"
                onChange={handleFileUpload}
              />
            </div>
          </SettingRow>
        )}
      </Card>

      <SectionLabel>Startup and System</SectionLabel>
      <Card>
        <SettingRow
          label="Launch at Startup"
          description="Start Expandly automatically when you log in"
        >
          <Toggle value={launchAtStartup} onChange={handleStartup} />
        </SettingRow>
        <SettingRow
          label="Minimise to Tray on Close"
          description="Keep Expandly running in the system tray when the window is closed"
        >
          <Toggle value={minimiseToTray} onChange={handleTray} />
        </SettingRow>
        <SettingRow
          label="Show in Taskbar"
          description="Show Expandly in the taskbar when running"
        >
          <Toggle value={showInTaskbar} onChange={handleTaskbar} />
        </SettingRow>
      </Card>
    </div>
  )
}

function AppearanceTab() {
  return (
    <div className="bg-gray-900 border border-gray-800 rounded-2xl px-6 py-8 flex flex-col items-center text-center gap-3">
      <Palette size={28} className="text-gray-600" />
      <p className="text-white font-medium">Themes coming soon</p>
      <p className="text-gray-500 text-sm">Theme support will be added in a future update once the app is complete.</p>
    </div>
  )
}

function DataTab() {
  const [importing, setImporting] = useState(false)
  const [resetting, setResetting] = useState(false)
  const [message, setMessage] = useState(null)
  const fileRef = useRef()

  const showMessage = (text, color = 'green') => {
    setMessage({ text, color })
    setTimeout(() => setMessage(null), 6000)
  }

  const handleExport = async () => {
    try {
      await invoke('export_config')
      showMessage('Config exported successfully')
    } catch (e) {
      showMessage('Export failed', 'red')
    }
  }

  const handleImport = async (e) => {
    const file = e.target.files[0]
    if (!file) return
    setImporting(true)
    try {
      const text = await file.text()
      const parsed = JSON.parse(text)
      await invoke('save_config', { newConfig: parsed })
      showMessage('Your new config has been imported successfully. Please restart the app to apply the changes.')
    } catch {
      showMessage('Import failed (invalid file)', 'red')
    } finally {
      setImporting(false)
    }
  }

  const handleReset = async () => {
    if (!window.confirm('This will delete all your snippets, triggers, hotkeys and variables. Are you sure?')) return
    setResetting(true)
    try {
      await invoke('reset_config')
      showMessage('All data reset to default')
    } catch {
      showMessage('Reset failed', 'red')
    } finally {
      setResetting(false)
    }
  }

  return (
    <div>
      {message && (
        <div className={`flex items-center gap-3 rounded-xl px-4 py-3 mb-6 ${message.color === 'red'
          ? 'bg-red-500/10 border border-red-500/30'
          : 'bg-emerald-500/10 border border-emerald-500/30'
          }`}>
          <CheckCircle size={16} className={message.color === 'red' ? 'text-red-400' : 'text-emerald-400'} />
          <p className={`text-sm ${message.color === 'red' ? 'text-red-300' : 'text-emerald-300'}`}>
            {message.text}
          </p>
        </div>
      )}

      <SectionLabel>Backup</SectionLabel>
      <Card>
        <SettingRow
          label="Export Config"
          description="Download a backup of all your snippets, triggers, hotkeys and variables"
        >
          <button
            onClick={handleExport}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-white text-sm transition-colors"
          >
            <Download size={14} />
            Export
          </button>
        </SettingRow>

        <SettingRow
          label="Import Config"
          description="Restore from a previously exported backup file"
        >
          <button
            onClick={() => fileRef.current.click()}
            disabled={importing}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-white text-sm transition-colors disabled:opacity-40"
          >
            <Upload size={14} />
            {importing ? 'Importing...' : 'Import'}
          </button>
          <input
            ref={fileRef}
            type="file"
            accept=".json"
            className="hidden"
            onChange={handleImport}
          />
        </SettingRow>
      </Card>

      <SectionLabel>Danger Zone</SectionLabel>
      <div className="bg-red-500/5 border border-red-500/20 rounded-2xl px-6">
        <SettingRow
          label="Reset All Data"
          description="Permanently delete all snippets, triggers, hotkeys and variables"
        >
          <button
            onClick={handleReset}
            disabled={resetting}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-red-600 hover:bg-red-500 text-white text-sm transition-colors disabled:opacity-40"
          >
            <Trash2 size={14} />
            {resetting ? 'Resetting...' : 'Reset'}
          </button>
        </SettingRow>
      </div>
    </div>
  )
}

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

function UpdatesTab() {
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

  const openUrl = (url) => window.open(url, '_blank')

  return (
    <div>
      <div className={`rounded-2xl border p-6 mb-4 transition-colors ${status === 'available' ? 'bg-orange-500/5 border-orange-500/30'
        : status === 'uptodate' ? 'bg-emerald-500/5 border-emerald-500/30'
          : status === 'error' ? 'bg-red-500/5 border-red-500/30'
            : 'bg-gray-900 border-gray-800'
        }`}>
        <div className="flex items-center justify-between gap-4">
          <div className="flex items-center gap-4">
            {status === 'checking' && (
              <RefreshCw size={24} className="text-blue-400 animate-spin shrink-0" />
            )}
            {status === 'uptodate' && (
              <CheckCircle size={24} className="text-emerald-400 shrink-0" />
            )}
            {status === 'available' && (
              <AlertCircle size={24} className="text-orange-400 shrink-0" />
            )}
            {status === 'error' && (
              <AlertCircle size={24} className="text-red-400 shrink-0" />
            )}
            {status === 'idle' && (
              <RefreshCw size={24} className="text-gray-600 shrink-0" />
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
                  <p className="text-white font-semibold">You are up to date</p>
                  <p className="text-gray-400 text-sm mt-0.5">Expandly {CURRENT_VERSION} is the latest version</p>
                </>
              )}
              {status === 'available' && (
                <>
                  <p className="text-white font-semibold">Update available - {release.tag_name}</p>
                  <p className="text-gray-400 text-sm mt-0.5">Released {formatDate(release.published_at)}</p>
                </>
              )}
              {status === 'error' && (
                <>
                  <p className="text-white font-semibold">Could not check for updates</p>
                  <p className="text-gray-400 text-sm mt-0.5">Check your internet connection</p>
                </>
              )}
            </div>
          </div>
          <button
            onClick={check}
            disabled={status === 'checking'}
            className="flex items-center gap-2 px-4 py-2 rounded-xl bg-gray-800 hover:bg-gray-700 text-white text-sm font-medium transition-colors disabled:opacity-40 shrink-0"
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
        <div>
          {release.body && (
            <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 mb-4">
              <h2 className="text-xs font-semibold text-gray-500 uppercase tracking-widest mb-4">
                What's New
              </h2>
              <div className="text-gray-300 text-sm leading-relaxed whitespace-pre-wrap">
                {release.body}
              </div>
            </div>
          )}

          {assets.length > 0 && (
            <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 mb-4">
              <h2 className="text-xs font-semibold text-gray-500 uppercase tracking-widest mb-4">
                Downloads
              </h2>
              <div className="flex flex-col gap-2">
                {assets.map((asset, i) => (
                  <div
                    key={i}
                    className="flex items-center justify-between px-4 py-3 bg-gray-800 rounded-xl"
                  >
                    <div className="flex items-center gap-3 min-w-0">
                      <Download size={15} className="text-gray-400 shrink-0" />
                      <span className="text-white text-sm truncate">{asset.name}</span>
                    </div>
                    <div className="flex items-center gap-3 shrink-0">
                      <span className="text-gray-500 text-xs">{formatBytes(asset.size)}</span>
                      <button
                        onClick={() => openUrl(asset.browser_download_url)}
                        className="text-blue-400 hover:text-blue-300 transition-colors"
                      >
                        <ExternalLink size={13} />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          <button
            onClick={() => openUrl(release.html_url)}
            className="flex items-center justify-center gap-2 w-full py-3 rounded-2xl bg-gray-900 border border-gray-800 hover:border-gray-700 text-gray-400 hover:text-white text-sm transition-colors"
          >
            <ExternalLink size={14} />
            View on GitHub
          </button>
        </div>
      )}
    </div>
  )
}

export default function Settings() {
  const [activeTab, setActiveTab] = useState('engine')

  return (
    <div className="max-w-3xl mx-auto">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-white">Settings</h1>
        <p className="text-gray-400 mt-1">Configure Expandly to your preference</p>
      </div>

      <div className="flex gap-1 bg-gray-900 border border-gray-800 rounded-xl p-1 mb-8">
        {tabs.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setActiveTab(id)}
            className={`flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${activeTab === id
              ? 'bg-blue-600 text-white'
              : 'text-gray-400 hover:text-white hover:bg-gray-800'
              }`}
          >
            <Icon size={15} />
            {label}
          </button>
        ))}
      </div>

      <div>
        {activeTab === 'engine' && <EngineTab />}
        {activeTab === 'appearance' && <AppearanceTab />}
        {activeTab === 'data' && <DataTab />}
        {activeTab === 'updates' && <UpdatesTab />}
      </div>
    </div>
  )
}