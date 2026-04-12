import { useEffect, useState } from 'react'
import { AlertCircle, X } from 'lucide-react'
import { useInvoke } from '../../hooks/useInvoke'
import { useConfig } from '../../hooks/useConfig'
import Toggle from '../../components/Toggle'
import { Card, SectionLabel, SettingRow } from './shared'

function AdvancedModal({ onClose }) {
  const invoke = useInvoke()
  const [expansionDelay, setExpansionDelay] = useState(325)
  const [hotkeyDelay, setHotkeyDelay] = useState(80)

  useEffect(() => {
    invoke('get_config').then(config => {
      setExpansionDelay(config.expansion_delay_ms ?? 325)
      setHotkeyDelay(config.hotkey_delay_ms ?? 80)
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
            <p className="text-gray-500 text-xs mt-0.5">Fine-tune expansion and hotkey injection delays.</p>
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
            <SettingRow label="Expansion Delay" description="Time in ms between keystroke deletion and text injection.">
              <div className="flex items-center gap-2">
                <input
                  type="number"
                  min="0"
                  max="2000"
                  value={expansionDelay}
                  onChange={event => {
                    const value = Math.max(0, parseInt(event.target.value) || 0)
                    setExpansionDelay(value)
                    save({ expansionDelayMs: value })
                  }}
                  className="w-20 bg-gray-800 border border-gray-700 rounded-lg px-3 py-1.5 text-white text-sm text-center focus:outline-none focus:border-blue-500 transition-colors"
                />
                <span className="text-gray-500 text-sm">ms</span>
              </div>
            </SettingRow>

            <SettingRow label="Hotkey Inject Delay" description="Time in ms to wait after a hotkey is pressed before pasting.">
              <div className="flex items-center gap-2">
                <input
                  type="number"
                  min="0"
                  max="2000"
                  value={hotkeyDelay}
                  onChange={event => {
                    const value = Math.max(0, parseInt(event.target.value) || 0)
                    setHotkeyDelay(value)
                    save({ hotkeyDelayMs: value })
                  }}
                  className="w-20 bg-gray-800 border border-gray-700 rounded-lg px-3 py-1.5 text-white text-sm text-center focus:outline-none focus:border-blue-500 transition-colors"
                />
                <span className="text-gray-500 text-sm">ms</span>
              </div>
            </SettingRow>
          </div>
        </div>

        <div className="px-6 py-4 border-t border-gray-800">
          <button onClick={onClose} className="w-full py-2.5 rounded-xl bg-gray-800 hover:bg-gray-700 text-white text-sm transition-colors">
            Done
          </button>
        </div>
      </div>
    </div>
  )
}

export default function SystemTab() {
  const invoke = useInvoke()
  const { config } = useConfig()

  const [enabled, setEnabled] = useState(false)
  const [launchAtStartup, setLaunchAtStartup] = useState(false)
  const [launchMinimised, setLaunchMinimised] = useState(false)
  const [minimiseToTray, setMinimiseToTray] = useState(false)
  const [appVersion, setAppVersion] = useState('')
  const [bufferSize, setBufferSize] = useState(16)
  const [clearBufferOnSwitch, setClearBufferOnSwitch] = useState(true)
  const [debugEnabled, setDebugEnabled] = useState(false)
  const [debugLevel, setDebugLevel] = useState('errors')
  const [showAdvanced, setShowAdvanced] = useState(false)

  useEffect(() => {
    if (!config) return
    setEnabled(config.enabled)
    setLaunchAtStartup(config.launch_at_startup ?? false)
    setMinimiseToTray(config.minimise_to_tray ?? false)
    setLaunchMinimised(config.launch_minimised ?? false)
    setBufferSize(config.buffer_size ?? 16)
    setClearBufferOnSwitch(config.clear_buffer_on_switch ?? true)
    setDebugEnabled(config.debug_enabled ?? false)
    setDebugLevel(config.debug_level ?? 'errors')
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

  const saveDebug = (overrides = {}) =>
    invoke('update_debug_settings', {
      debugEnabled: overrides.debugEnabled ?? debugEnabled,
      debugLevel: overrides.debugLevel ?? debugLevel,
    })

  const handleBufferSize = value => {
    const nextValue = Math.max(1, Math.min(64, parseInt(value) || 16))
    setBufferSize(nextValue)
    invoke('update_buffer_size', { bufferSize: nextValue })
  }

  const handleClearBuffer = value => {
    setClearBufferOnSwitch(value)
    invoke('update_performance_settings', { hotkeyDelayMs: config?.hotkey_delay_ms ?? 80, clearBufferOnSwitch: value })
  }

  return (
    <div>
      <SectionLabel>Engine</SectionLabel>
      <Card>
        <SettingRow label="Enable Engine" description="Master switch - when off, no triggers or hotkeys will fire">
          <Toggle value={enabled} onChange={value => { setEnabled(value); saveEngine({ enabled: value }) }} />
        </SettingRow>
        <div className="py-3">
          <p className="text-xs text-gray-600">Expandly Engine {appVersion}</p>
        </div>
      </Card>

      <SectionLabel>Startup</SectionLabel>
      <Card>
        <SettingRow label="Launch at Login" description="Start Expandly automatically when you log in">
          <Toggle value={launchAtStartup} onChange={value => { setLaunchAtStartup(value); saveSystem({ launchAtStartup: value }) }} />
        </SettingRow>
        <SettingRow label="Launch Minimised" description="Start Expandly minimised to the system tray">
          <Toggle value={launchMinimised} onChange={value => { setLaunchMinimised(value); saveSystem({ launchMinimised: value }) }} />
        </SettingRow>
        <SettingRow label="Minimise to Tray on Close" description="Keep Expandly running in the system tray when the window is closed">
          <Toggle value={minimiseToTray} onChange={value => { setMinimiseToTray(value); saveSystem({ minimiseToTray: value }) }} />
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
            onChange={event => handleBufferSize(event.target.value)}
            className="bg-gray-800 border border-gray-700 rounded-lg px-3 py-1.5 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors"
          >
            {[16, 24, 32, 64].map(value => (
              <option key={value} value={value}>{value} chars</option>
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

        <SettingRow label="Advanced Performance Settings" description="Configure advanced performance related settings.">
          <button onClick={() => setShowAdvanced(true)} className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-white text-sm transition-colors">
            Open
          </button>
        </SettingRow>
      </Card>

      <SectionLabel>Debug</SectionLabel>
      <Card>
        <SettingRow label="Debug Mode" description="Creates by day log files that are automatically deleted after 7 days when enabled.">
          <Toggle value={debugEnabled} onChange={value => { setDebugEnabled(value); saveDebug({ debugEnabled: value }) }} />
        </SettingRow>
        {debugEnabled && (
          <>
            <SettingRow label="Log Level" description="Controls how much detail is written to the log file.">
              <select
                value={debugLevel}
                onChange={event => { setDebugLevel(event.target.value); saveDebug({ debugLevel: event.target.value }) }}
                className="bg-gray-800 border border-gray-700 rounded-lg px-3 py-1.5 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors"
              >
                <option value="errors">Errors</option>
                <option value="warnings">Errors & Warnings</option>
                <option value="verbose">Verbose</option>
              </select>
            </SettingRow>
            <SettingRow label="Debug Logs Folder" description="Open the folder where Expandly stores its daily debug log files.">
              <button
                onClick={() => invoke('open_debug_logs_folder').catch(() => { })}
                className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-white text-sm transition-colors"
              >
                Open Folder
              </button>
            </SettingRow>
          </>
        )}
      </Card>

      {showAdvanced && <AdvancedModal onClose={() => setShowAdvanced(false)} />}
    </div>
  )
}
