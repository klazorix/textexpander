import { useState, useEffect, useRef } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { Settings as SettingsIcon, Palette, Database, RefreshCw, Upload, X, Volume2, CheckCircle, AlertCircle, Download, ExternalLink, Trash2 } from 'lucide-react'
import { useInvoke } from '../hooks/useInvoke'
import { useConfig } from '../hooks/useConfig'
import Toggle from '../components/Toggle'
import ConfirmModal from '../components/ConfirmModal'

const tabs = [
  { id: 'engine', label: 'System', icon: SettingsIcon },
  { id: 'appearance', label: 'Customise', icon: Palette },
  { id: 'data', label: 'Data', icon: Database },
  { id: 'updates', label: 'Updates', icon: RefreshCw },
]

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

// ── Advanced Modal ────────────────────────────────────────────────────────

function AdvancedModal({ onClose }) {
  const invoke = useInvoke()
  const [expansionDelay, setExpansionDelay] = useState(325)
  const [hotkeyDelay, setHotkeyDelay] = useState(80)

  useEffect(() => {
    invoke('get_config').then(c => {
      setExpansionDelay(c.expansion_delay_ms ?? 325)
      setHotkeyDelay(c.hotkey_delay_ms ?? 80)
    })
  }, [])

  const save = (overrides = {}) => {
    invoke('update_expansion_delay', { expansionDelayMs: overrides.expansionDelayMs ?? expansionDelay })
    invoke('update_performance_settings', {
      hotkeyDelayMs: overrides.hotkeyDelayMs ?? hotkeyDelay,
      clearBufferOnSwitch: true,
    })
  }

  return (
    <div className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 p-6">
      <div className="bg-gray-900 border border-gray-700 rounded-2xl w-full max-w-lg shadow-2xl">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-800">
          <div>
            <h2 className="text-white font-semibold">Advanced Settings</h2>
            <p className="text-gray-500 text-xs mt-0.5">Debugging and advanced performance settings.</p>
          </div>
          <button onClick={onClose} className="text-gray-500 hover:text-white transition-colors">
            <X size={20} />
          </button>
        </div>

        <div className="px-6 py-4">
          <div className="bg-amber-500/10 border border-amber-500/30 rounded-xl px-4 py-3 mb-6 flex items-center justify-center gap-3">
            <AlertCircle size={15} className="text-amber-400 shrink-0" />
            <p className="text-amber-300 text-xs">Only users who fully understand the potential impact should modify these settings.</p>
          </div>

          <div className="bg-gray-900 border border-gray-800 rounded-2xl px-6">
            <SettingRow
              label="Expansion Delay"
              description="Time in ms between keystroke deletion and text injection."
            >
              <div className="flex items-center gap-2">
                <input
                  type="number"
                  min="0"
                  max="2000"
                  value={expansionDelay}
                  onChange={e => {
                    const val = Math.max(0, parseInt(e.target.value) || 0)
                    setExpansionDelay(val)
                    save({ expansionDelayMs: val })
                  }}
                  className="w-20 bg-gray-800 border border-gray-700 rounded-lg px-3 py-1.5 text-white text-sm text-center focus:outline-none focus:border-blue-500 transition-colors"
                />
                <span className="text-gray-500 text-sm">ms</span>
              </div>
            </SettingRow>

            <SettingRow
              label="Hotkey Inject Delay"
              description="Time in ms to wait after a hotkey is pressed before pasting."
            >
              <div className="flex items-center gap-2">
                <input
                  type="number"
                  min="0"
                  max="2000"
                  value={hotkeyDelay}
                  onChange={e => {
                    const val = Math.max(0, parseInt(e.target.value) || 0)
                    setHotkeyDelay(val)
                    save({ hotkeyDelayMs: val })
                  }}
                  className="w-20 bg-gray-800 border border-gray-700 rounded-lg px-3 py-1.5 text-white text-sm text-center focus:outline-none focus:border-blue-500 transition-colors"
                />
                <span className="text-gray-500 text-sm">ms</span>
              </div>
            </SettingRow>
          </div>
        </div>

        <div className="px-6 py-4 border-t border-gray-800">
          <button
            onClick={onClose}
            className="w-full py-2.5 rounded-xl bg-gray-800 hover:bg-gray-700 text-white text-sm transition-colors"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  )
}

// ── Engine Tab ────────────────────────────────────────────────────────────

function EngineTab() {
  const invoke = useInvoke()
  const { config } = useConfig()

  const [enabled, setEnabled] = useState(false)
  const [launchAtStartup, setLaunchAtStartup] = useState(false)
  const [launchMinimised, setLaunchMinimised] = useState(false)
  const [minimiseToTray, setMinimiseToTray] = useState(false)
  const [appVersion, setAppVersion] = useState('')
  const [bufferSize, setBufferSize] = useState(16)
  const [clearBufferOnSwitch, setClearBufferOnSwitch] = useState(true)
  const [showAdvanced, setShowAdvanced] = useState(false)

  useEffect(() => {
    if (!config) return
    setEnabled(config.enabled)
    setLaunchAtStartup(config.launch_at_startup ?? false)
    setMinimiseToTray(config.minimise_to_tray ?? false)
    setLaunchMinimised(config.launch_minimised ?? false)
    setBufferSize(config.buffer_size ?? 16)
    setClearBufferOnSwitch(config.clear_buffer_on_switch ?? true)
  }, [config])

  useEffect(() => {
    invoke('get_app_version').then(setAppVersion)
  }, [])

  const saveEngine = (overrides = {}) =>
    invoke('update_engine_settings', {
      enabled: overrides.enabled ?? enabled,
      soundEnabled: config?.sound_enabled ?? false,
      soundPath: config?.sound_path ?? null,
    })

  const saveSystem = (overrides = {}) =>
    invoke('update_system_settings', {
      launchAtStartup: overrides.launchAtStartup ?? launchAtStartup,
      launchMinimised: overrides.launchMinimised ?? launchMinimised,
      minimiseToTray: overrides.minimiseToTray ?? minimiseToTray,
    })

  const handleToggleEngine = (val) => { setEnabled(val); saveEngine({ enabled: val }) }
  const handleStartup = (val) => { setLaunchAtStartup(val); saveSystem({ launchAtStartup: val }) }
  const handleTray = (val) => { setMinimiseToTray(val); saveSystem({ minimiseToTray: val }) }
  const handleLaunchMinimised = (val) => { setLaunchMinimised(val); saveSystem({ launchMinimised: val }) }

  const handleBufferSize = (val) => {
    const num = Math.max(1, Math.min(64, parseInt(val) || 16))
    setBufferSize(num)
    invoke('update_buffer_size', { bufferSize: num })
  }

  const handleClearBuffer = (val) => {
    setClearBufferOnSwitch(val)
    invoke('update_performance_settings', { hotkeyDelayMs: config?.hotkey_delay_ms ?? 80, clearBufferOnSwitch: val })
  }

  return (
    <div>
      <SectionLabel>Engine</SectionLabel>
      <Card>
        <SettingRow label="Enable Engine" description="Master switch - when off, no triggers or hotkeys will fire">
          <Toggle value={enabled} onChange={handleToggleEngine} />
        </SettingRow>
        <div className="py-3">
          <p className="text-xs text-gray-600">Expandly Engine {appVersion}</p>
        </div>
      </Card>

      <SectionLabel>Startup</SectionLabel>
      <Card>
        <SettingRow label="Launch at Login" description="Start Expandly automatically when you log in">
          <Toggle value={launchAtStartup} onChange={handleStartup} />
        </SettingRow>
        <SettingRow label="Launch Minimised" description="Start Expandly minimised to the system tray">
          <Toggle value={launchMinimised} onChange={handleLaunchMinimised} />
        </SettingRow>
        <SettingRow label="Minimise to Tray on Close" description="Keep Expandly running in the system tray when the window is closed">
          <Toggle value={minimiseToTray} onChange={handleTray} />
        </SettingRow>
      </Card>

      <SectionLabel>Performance</SectionLabel>
      <Card>
        <SettingRow
          label={
            <div className="flex items-center gap-2">
              <span>Buffer Size</span>
              <span className="text-xs bg-orange-500/15 text-orange-300 px-2 py-0.5 rounded-md">Medium Impact</span>
            </div>
          }
          description="Maximum number of characters tracked for trigger matching (indirectly sets the maximum trigger length)."
        >
          <select
            value={bufferSize}
            onChange={e => handleBufferSize(e.target.value)}
            className="bg-gray-800 border border-gray-700 rounded-lg px-3 py-1.5 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors"
          >
            {[16, 24, 32, 64].map(v => (
              <option key={v} value={v}>{v} chars</option>
            ))}
          </select>
        </SettingRow>

        <SettingRow
          label={
            <div className="flex items-center gap-2">
              <span>Clear Buffer on Window Switch</span>
              <span className="text-xs bg-red-500/15 text-red-300 px-2 py-0.5 rounded-md">High Impact</span>
            </div>
          }
          description="Clears the typed character buffer when you switch applications, preventing accidental trigger matches."
        >
          <Toggle value={clearBufferOnSwitch} onChange={handleClearBuffer} />
        </SettingRow>
      </Card>

      <SectionLabel>Advanced</SectionLabel>
      <Card>
        <SettingRow label="Advanced Settings" description="Debugging and advanced performance settings.">
          <button
            onClick={() => setShowAdvanced(true)}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-white text-sm transition-colors"
          >
            Open
          </button>
        </SettingRow>
      </Card>

      {showAdvanced && <AdvancedModal onClose={() => setShowAdvanced(false)} />}
    </div>
  )
}

// ── Appearance Tab ────────────────────────────────────────────────────────

function AppearanceTab() {
  const invoke = useInvoke()
  const { config } = useConfig()
  const fileRef = useRef()

  const [soundEnabled, setSoundEnabled] = useState(false)
  const [soundPath, setSoundPath] = useState(null)
  const [soundName, setSoundName] = useState(null)

  useEffect(() => {
    if (!config) return
    setSoundEnabled(config.sound_enabled)
    setSoundPath(config.sound_path ?? null)
    if (config.sound_path) setSoundName(config.sound_path.split(/[\\/]/).pop())
  }, [config])

  const saveEngine = (overrides = {}) =>
    invoke('update_engine_settings', {
      enabled: config?.enabled ?? true,
      soundEnabled: overrides.soundEnabled ?? soundEnabled,
      soundPath: overrides.soundPath !== undefined ? overrides.soundPath : soundPath,
    })

  const handleToggleSound = (val) => { setSoundEnabled(val); saveEngine({ soundEnabled: val }) }

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
      <SectionLabel>Themes</SectionLabel>
      <div className="bg-gray-900 border border-gray-800 rounded-2xl px-6 py-8 flex flex-col items-center text-center gap-3 mb-4">
        <Palette size={28} className="text-gray-600" />
        <p className="text-white font-medium">Themes coming soon</p>
        <p className="text-gray-500 text-sm">Theme support will be added in a future update once the app is complete.</p>
      </div>

      <SectionLabel>Expansion Sound</SectionLabel>
      <Card>
        <SettingRow label="Play Sound on Expansion" description="Plays a custom sound every time a snippet expands">
          <Toggle value={soundEnabled} onChange={handleToggleSound} />
        </SettingRow>
        {soundEnabled && (
          <SettingRow label="Sound File" description="Upload a .mp3 or .wav to play on expansion. Maximum length of 10 seconds.">
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
                  <button onClick={handleRemoveSound} className="p-1.5 rounded-lg text-gray-500 hover:text-red-400 hover:bg-gray-800 transition-colors">
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
              <input ref={fileRef} type="file" accept=".mp3,.wav" className="hidden" onChange={handleFileUpload} />
            </div>
          </SettingRow>
        )}
      </Card>
    </div>
  )
}

// ── Data Tab ──────────────────────────────────────────────────────────────

function DataTab() {
  const invoke = useInvoke()
  const { config } = useConfig()
  const fileRef = useRef()

  const [importing, setImporting] = useState(false)
  const [resetting, setResetting] = useState(false)
  const [message, setMessage] = useState(null)
  const [trackStats, setTrackStats] = useState(true)
  const [confirmModal, setConfirmModal] = useState(null)

  useEffect(() => {
    if (!config) return
    setTrackStats(config.track_stats ?? true)
  }, [config])

  const showMessage = (text, color = 'green') => {
    setMessage({ text, color })
    setTimeout(() => setMessage(null), 6000)
  }

  const handleTrackStats = (val) => {
    if (!val) {
      setConfirmModal({
        message: 'Disabling statistics tracking will permanently clear all recorded statistics. This cannot be undone. Are you sure?',
        onConfirm: async () => {
          setConfirmModal(null)
          await invoke('reset_stats')
          setTrackStats(false)
          await invoke('update_track_stats', { trackStats: false })
        },
        onCancel: () => setConfirmModal(null),
      })
      return
    }
    setTrackStats(val)
    invoke('update_track_stats', { trackStats: val })
  }

  const handleExport = async () => {
    try {
      await invoke('export_config')
      showMessage('Config exported successfully')
    } catch {
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

  const handleReset = () => {
    setConfirmModal({
      message: 'This will permanently delete all your snippets, triggers, hotkeys and variables. This cannot be undone. Are you sure?',
      onConfirm: async () => {
        setConfirmModal(null)
        setResetting(true)
        try {
          await invoke('reset_config')
          showMessage('All data reset to default')
        } catch {
          showMessage('Reset failed', 'red')
        } finally {
          setResetting(false)
        }
      },
      onCancel: () => setConfirmModal(null),
    })
  }

  return (
    <div>
      {message && (
        <div className={`flex items-center gap-3 rounded-xl px-4 py-3 mb-6 ${message.color === 'red' ? 'bg-red-500/10 border border-red-500/30' : 'bg-emerald-500/10 border border-emerald-500/30'}`}>
          <CheckCircle size={16} className={message.color === 'red' ? 'text-red-400' : 'text-emerald-400'} />
          <p className={`text-sm ${message.color === 'red' ? 'text-red-300' : 'text-emerald-300'}`}>{message.text}</p>
        </div>
      )}

      <SectionLabel>Statistics</SectionLabel>
      <Card>
        <SettingRow label="Track Statistics" description="Record expansion counts and characters saved over time">
          <Toggle value={trackStats} onChange={handleTrackStats} />
        </SettingRow>
      </Card>

      <SectionLabel>Backup</SectionLabel>
      <Card>
        <SettingRow label="Export Config" description="Download a backup of all your snippets, triggers, hotkeys and variables">
          <button onClick={handleExport} className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-white text-sm transition-colors">
            <Upload size={14} />
            Export
          </button>
        </SettingRow>
        <SettingRow label="Import Config" description="Restore from a previously exported backup file">
          <button
            onClick={() => fileRef.current.click()}
            disabled={importing}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-white text-sm transition-colors disabled:opacity-40"
          >
            <Download size={14} />
            {importing ? 'Importing...' : 'Import'}
          </button>
          <input ref={fileRef} type="file" accept=".json" className="hidden" onChange={handleImport} />
        </SettingRow>
      </Card>

      <SectionLabel>Danger Zone</SectionLabel>
      <div className="bg-red-500/5 border border-red-500/20 rounded-2xl px-6">
        <SettingRow label="Reset All Data" description="Permanently delete all snippets, triggers, hotkeys and variables">
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

      {confirmModal && (
        <ConfirmModal
          message={confirmModal.message}
          onConfirm={confirmModal.onConfirm}
          onCancel={confirmModal.onCancel}
        />
      )}
    </div>
  )
}

// ── Updates Tab ───────────────────────────────────────────────────────────

function newerVersion(latest, current) {
  const clean = v => v.replace(/^v/, '')
  const parse = v => {
    const str = clean(v)
    const match = str.match(/^(\d+\.\d+\.\d+)b(\d+)$/)
    if (match) return { parts: match[1].split('.').map(Number), pre: parseInt(match[2]) }
    return { parts: str.split('.').map(Number), pre: null }
  }
  const a = parse(latest)
  const b = parse(current)
  for (let i = 0; i < 3; i++) {
    if ((a.parts[i] ?? 0) > (b.parts[i] ?? 0)) return true
    if ((a.parts[i] ?? 0) < (b.parts[i] ?? 0)) return false
  }
  if (a.pre === null && b.pre !== null) return true
  if (a.pre !== null && b.pre === null) return false
  if (a.pre !== null && b.pre !== null) return a.pre > b.pre
  return false
}

function formatDate(iso) {
  return new Date(iso).toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric' })
}

function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function ChangelogModal({ release, onClose }) {
  const invoke = useInvoke()
  const openUrl = (url) => invoke('open_url', { url }).catch(() => { })

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

function UpdatesTab() {
  const invoke = useInvoke()
  const [status, setStatus] = useState('idle')
  const [latestRelease, setLatestRelease] = useState(null)
  const [currentRelease, setCurrentRelease] = useState(null)
  const [checkedAt, setCheckedAt] = useState(null)
  const [appVersion, setAppVersion] = useState('')
  const [showChangelog, setShowChangelog] = useState(null)

  useEffect(() => {
    invoke('get_app_version').then(v => {
      setAppVersion(v)
      checkForUpdates(v)
    })
  }, [])

  const checkForUpdates = async (version) => {
    setStatus('checking')
    try {
      const allRes = await fetch('https://api.github.com/repos/klazorix/Expandly/releases')
      if (!allRes.ok) throw new Error()
      const allReleases = await allRes.json()
      const current = allReleases.find(r => r.tag_name === version || r.tag_name === `v${version}`)
      setCurrentRelease(current ?? null)

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
              {status === 'available' && (
                <>
                  <p className="text-white font-semibold">Update available — {latestRelease.tag_name}</p>
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

      {showChangelog && (
        <ChangelogModal release={showChangelog} onClose={() => setShowChangelog(null)} />
      )}
    </div>
  )
}

// ── Root ──────────────────────────────────────────────────────────────────

export default function Settings() {
  const [activeTab, setActiveTab] = useState('engine')

  // Support navigating directly to a tab via router state
  useEffect(() => {
    try {
      const state = window.history.state?.usr
      if (state?.tab) setActiveTab(state.tab)
    } catch { }
  }, [])

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
            className={`flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
              activeTab === id ? 'bg-blue-600 text-white' : 'text-gray-400 hover:text-white hover:bg-gray-800'
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
