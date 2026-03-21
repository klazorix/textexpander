import { useEffect, useState } from 'react'
import { Plus, Pencil, Trash2, Check, Keyboard } from 'lucide-react'
import { useInvoke } from '../hooks/useInvoke'
import { useConfig } from '../hooks/useConfig'
import Modal from '../components/Modal'

function KeyRecorder({ value, onChange }) {
  const [recording, setRecording] = useState(false)
  const [current, setCurrent] = useState(value || '')

  useEffect(() => {
    if (!recording) return
    const held = new Set()
    const onDown = (e) => {
      e.preventDefault()
      const modifiers = ['Control', 'Shift', 'Alt', 'Meta']
      const key = e.key === ' ' ? 'Space' : e.key === 'Meta' ? 'Super' : e.key
      held.add(key)
      const mods = modifiers
        .map(m => m === 'Meta' ? 'Super' : m)
        .filter(m => held.has(m))
      const regular = [...held].filter(k => !['Control', 'Shift', 'Alt', 'Super'].includes(k))
      const combo = [...mods, ...regular].join('+')
      if (combo) setCurrent(combo)
    }
    window.addEventListener('keydown', onDown)
    return () => window.removeEventListener('keydown', onDown)
  }, [recording])

  const confirm = () => { onChange(current); setRecording(false) }
  const cancel = () => { setCurrent(value || ''); setRecording(false) }

  if (recording) {
    return (
      <div className="flex flex-col gap-2">
        <div className="w-full px-3 py-2 rounded-lg border border-blue-500 bg-blue-500/10 text-blue-400 text-sm font-mono animate-pulse min-h-9">
          {current || 'Hold your keys...'}
        </div>
        <div className="flex gap-2">
          <button
            onClick={confirm}
            disabled={!current}
            className="flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium transition-colors disabled:opacity-40"
          >
            <Check size={14} />
            Confirm
          </button>
          <button onClick={cancel} className="px-3 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-400 hover:text-white text-sm transition-colors">
            Cancel
          </button>
        </div>
      </div>
    )
  }

  return (
    <button
      onClick={() => setRecording(true)}
      className="w-full text-left px-3 py-2 rounded-lg border border-gray-700 bg-gray-800 text-white hover:border-gray-600 transition-colors text-sm font-mono"
    >
      {current || 'Click to record'}
    </button>
  )
}

function HotkeyForm({ form, onChange, snippets }) {
  return (
    <>
      <div>
        <label className="text-xs text-gray-400 mb-1 block">Key Combo</label>
        <KeyRecorder value={form.keys} onChange={keys => onChange({ ...form, keys })} />
      </div>
      <div>
        <label className="text-xs text-gray-400 mb-1 block">Pastes Snippet</label>
        <select
          className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors"
          value={form.expansion_id}
          onChange={e => onChange({ ...form, expansion_id: e.target.value })}
        >
          <option value="">Select a snippet...</option>
          {snippets.map(s => (
            <option key={s.id} value={s.id}>{s.name}</option>
          ))}
        </select>
      </div>
    </>
  )
}

export default function Hotkeys() {
  const invoke = useInvoke()
  const { config, reload } = useConfig()

  const hotkeys = config?.hotkeys ?? []
  const snippets = config ? Object.values(config.expansions) : []

  const blank = { keys: '', expansion_id: '' }
  const [showAdd, setShowAdd] = useState(false)
  const [editing, setEditing] = useState(null)
  const [addForm, setAddForm] = useState(blank)
  const [editForm, setEditForm] = useState(blank)

  const snippetName = id => snippets.find(s => s.id === id)?.name ?? 'Unknown'

  const handleAdd = async () => {
    await invoke('create_hotkey', { keys: addForm.keys, expansionId: addForm.expansion_id })
    setShowAdd(false)
    setAddForm(blank)
    reload()
  }

  const handleEdit = async () => {
    await invoke('update_hotkey', { id: editing.id, keys: editForm.keys, expansionId: editForm.expansion_id })
    setEditing(null)
    reload()
  }

  const handleDelete = async (id) => {
    await invoke('delete_hotkey', { id })
    reload()
  }

  const openEdit = (h) => {
    setEditForm({ keys: h.keys, expansion_id: h.expansion_id })
    setEditing(h)
  }

  const formatKeys = (combo) =>
    combo.split('+').map((k, i) => (
      <span key={i} className="inline-flex items-center px-2 py-0.5 bg-gray-800 border border-gray-700 rounded text-xs text-gray-300 font-mono">
        {k}
      </span>
    ))

  return (
    <div className="max-w-4xl mx-auto">
      <div className="flex items-center justify-between mb-10">
        <div>
          <h1 className="text-3xl font-bold text-white">Hotkeys</h1>
          <p className="text-gray-400 mt-1">{hotkeys.length} hotkey{hotkeys.length !== 1 ? 's' : ''} configured</p>
        </div>
        <button
          onClick={() => { setAddForm(blank); setShowAdd(true) }}
          className="flex items-center gap-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium px-4 py-2 rounded-xl transition-colors"
        >
          <Plus size={16} />
          New Hotkey
        </button>
      </div>

      {hotkeys.length === 0 && (
        <div className="flex flex-col items-center justify-center py-24 text-center">
          <div className="bg-gray-900 border border-gray-800 rounded-2xl p-5 mb-4">
            <Keyboard size={28} className="text-gray-600" />
          </div>
          <p className="text-gray-400 font-medium">No hotkeys yet</p>
          <p className="text-gray-600 text-sm mt-1">Click "New Hotkey" to create your first one</p>
        </div>
      )}

      <div className="flex flex-col gap-3">
        {hotkeys.map(h => (
          <div key={h.id} className="bg-gray-900 border border-gray-800 rounded-2xl px-5 py-4 flex items-center justify-between gap-4 group hover:border-gray-700 transition-colors">
            <div className="flex items-center gap-3 min-w-0">
              <div className="flex items-center gap-1 shrink-0">{formatKeys(h.keys)}</div>
              <span className="text-gray-500 text-sm">→</span>
              <span className="text-white text-sm truncate">{snippetName(h.expansion_id)}</span>
            </div>
            <div className="flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
              <button onClick={() => openEdit(h)} className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-gray-800 transition-colors">
                <Pencil size={15} />
              </button>
              <button onClick={() => handleDelete(h.id)} className="p-2 rounded-lg text-gray-400 hover:text-red-400 hover:bg-gray-800 transition-colors">
                <Trash2 size={15} />
              </button>
            </div>
          </div>
        ))}
      </div>

      {showAdd && (
        <Modal
          title="New Hotkey"
          onClose={() => setShowAdd(false)}
          onSave={handleAdd}
          disabled={!addForm.keys.trim() || !addForm.expansion_id}
        >
          <HotkeyForm form={addForm} onChange={setAddForm} snippets={snippets} />
        </Modal>
      )}

      {editing && (
        <Modal
          title="Edit Hotkey"
          onClose={() => setEditing(null)}
          onSave={handleEdit}
          disabled={!editForm.keys.trim() || !editForm.expansion_id}
        >
          <HotkeyForm form={editForm} onChange={setEditForm} snippets={snippets} />
        </Modal>
      )}
    </div>
  )
}
