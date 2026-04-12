import { useEffect, useRef, useState } from 'react'
import { CheckCircle, Download, Trash2, Upload } from 'lucide-react'
import { useInvoke } from '../../hooks/useInvoke'
import { useConfig } from '../../hooks/useConfig'
import Toggle from '../../components/Toggle'
import ConfirmModal from '../../components/ConfirmModal'
import { Card, SectionLabel, SettingRow } from './shared'

export default function DataTab() {
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

  const closeConfirmModal = () => setConfirmModal(null)

  const handleTrackStats = async value => {
    if (value) {
      setTrackStats(true)
      invoke('update_track_stats', { trackStats: true })
      return
    }

    setConfirmModal({
      message: 'Disabling statistics tracking will permanently clear all recorded statistics. This cannot be undone. Are you sure?',
      onConfirm: async () => {
        closeConfirmModal()
        await invoke('reset_stats')
        setTrackStats(false)
        await invoke('update_track_stats', { trackStats: false })
      },
      onCancel: closeConfirmModal,
    })
  }

  const handleExport = async () => {
    try {
      await invoke('export_config')
      showMessage('Config exported successfully')
    } catch {
      showMessage('Export failed', 'red')
    }
  }

  const handleImport = async event => {
    const file = event.target.files[0]
    if (!file) return

    setImporting(true)
    try {
      const parsed = JSON.parse(await file.text())
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
        closeConfirmModal()
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
      onCancel: closeConfirmModal,
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
