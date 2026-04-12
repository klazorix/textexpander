import { useEffect, useRef, useState } from 'react'
import { Palette, Upload, Volume2, X } from 'lucide-react'
import { useInvoke } from '../../hooks/useInvoke'
import { useConfig } from '../../hooks/useConfig'
import Toggle from '../../components/Toggle'
import { Card, SectionLabel, SettingRow } from './shared'

export default function CustomiseTab() {
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

  const handleToggleSound = value => {
    setSoundEnabled(value)
    saveEngine({ soundEnabled: value })
  }

  const handleFileUpload = async event => {
    const file = event.target.files[0]
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
