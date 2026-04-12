import { useState } from 'react'
import { Plus, Pencil, Trash2, Zap } from 'lucide-react'
import { useInvoke } from '../hooks/useInvoke'
import { useConfig } from '../hooks/useConfig'
import Modal from '../components/Modal'

const blankForm = { key: '', expansion_id: '', word_boundary: true }

function TriggerForm({ form, onChange, snippets, maxKeyLength }) {
  return (
    <>
      <div>
        <label className="text-xs text-gray-400 mb-1 block">Trigger Text</label>
        <input
          className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors font-mono"
          placeholder="e.g. /hi"
          value={form.key}
          maxLength={maxKeyLength}
          onChange={e => onChange({ ...form, key: e.target.value })}
        />
        <p className={`text-xs mt-1 text-right ${form.key.length >= maxKeyLength ? 'text-red-400' : 'text-gray-600'}`}>
          {form.key.length}/{maxKeyLength}
        </p>
      </div>

      <div>
        <label className="text-xs text-gray-400 mb-1 block">Expands To</label>
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

      <div className="flex items-center justify-between bg-gray-800 border border-gray-700 rounded-lg px-4 py-3">
        <div>
          <p className="text-sm text-white font-medium">Word Boundary</p>
          <p className="text-xs text-gray-500 mt-0.5">Only trigger when preceded by a space</p>
        </div>
        <button
          onClick={() => onChange({ ...form, word_boundary: !form.word_boundary })}
          className={`w-11 h-6 rounded-full transition-colors relative ${form.word_boundary ? 'bg-blue-600' : 'bg-gray-600'}`}
        >
          <span className={`absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-all ${form.word_boundary ? 'left-5' : 'left-0.5'}`} />
        </button>
      </div>
    </>
  )
}

export default function Triggers() {
  const invoke = useInvoke()
  const { config, reload } = useConfig()

  const triggers = config?.triggers ?? []
  const snippets = config ? Object.values(config.expansions) : []
  const maxKeyLength = config?.buffer_size ?? 16

  const [showAdd, setShowAdd] = useState(false)
  const [editing, setEditing] = useState(null)
  const [addForm, setAddForm] = useState(blankForm)
  const [editForm, setEditForm] = useState(blankForm)

  const snippetName = id => snippets.find(s => s.id === id)?.name ?? 'Unknown'

  const handleAdd = async () => {
    await invoke('create_trigger', { key: addForm.key, expansionId: addForm.expansion_id, wordBoundary: addForm.word_boundary })
    setShowAdd(false)
    setAddForm(blankForm)
    reload()
  }

  const handleEdit = async () => {
    await invoke('update_trigger', { id: editing.id, key: editForm.key, expansionId: editForm.expansion_id, wordBoundary: editForm.word_boundary })
    setEditing(null)
    reload()
  }

  const handleDelete = async (id) => {
    await invoke('delete_trigger', { id })
    reload()
  }

  const openEdit = (t) => {
    setEditForm({ key: t.key, expansion_id: t.expansion_id, word_boundary: t.word_boundary })
    setEditing(t)
  }

  return (
    <div className="max-w-4xl mx-auto">
      <div className="flex items-center justify-between mb-10">
        <div>
          <h1 className="text-3xl font-bold text-white">Triggers</h1>
          <p className="text-gray-400 mt-1">{triggers.length} trigger{triggers.length !== 1 ? 's' : ''} configured</p>
        </div>
        <button
          onClick={() => { setAddForm(blankForm); setShowAdd(true) }}
          className="flex items-center gap-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium px-4 py-2 rounded-xl transition-colors"
        >
          <Plus size={16} />
          New Trigger
        </button>
      </div>

      {triggers.length === 0 && (
        <div className="flex flex-col items-center justify-center py-24 text-center">
          <div className="bg-gray-900 border border-gray-800 rounded-2xl p-5 mb-4">
            <Zap size={28} className="text-gray-600" />
          </div>
          <p className="text-gray-400 font-medium">No triggers yet</p>
          <p className="text-gray-600 text-sm mt-1">Click "New Trigger" to create your first one</p>
        </div>
      )}

      <div className="flex flex-col gap-3">
        {triggers.map(t => (
          <div key={t.id} className="bg-gray-900 border border-gray-800 rounded-2xl px-5 py-4 flex items-center justify-between gap-4 group hover:border-gray-700 transition-colors">
            <div className="flex items-center gap-4 min-w-0">
              <span className="font-mono text-blue-400 bg-blue-500/10 px-3 py-1 rounded-lg text-sm shrink-0">{t.key}</span>
              <span className="text-gray-500 text-sm">→</span>
              <span className="text-white text-sm truncate">{snippetName(t.expansion_id)}</span>
              {t.word_boundary && (
                <span className="text-xs text-gray-600 bg-gray-800 px-2 py-0.5 rounded-full shrink-0">Word Boundary</span>
              )}
            </div>
            <div className="flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
              <button onClick={() => openEdit(t)} className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-gray-800 transition-colors">
                <Pencil size={15} />
              </button>
              <button onClick={() => handleDelete(t.id)} className="p-2 rounded-lg text-gray-400 hover:text-red-400 hover:bg-gray-800 transition-colors">
                <Trash2 size={15} />
              </button>
            </div>
          </div>
        ))}
      </div>

      {showAdd && (
        <Modal
          title="New Trigger"
          onClose={() => setShowAdd(false)}
          onSave={handleAdd}
          disabled={!addForm.key.trim() || !addForm.expansion_id}
        >
          <TriggerForm form={addForm} onChange={setAddForm} snippets={snippets} maxKeyLength={maxKeyLength} />
        </Modal>
      )}

      {editing && (
        <Modal
          title="Edit Trigger"
          onClose={() => setEditing(null)}
          onSave={handleEdit}
          disabled={!editForm.key.trim() || !editForm.expansion_id}
        >
          <TriggerForm form={editForm} onChange={setEditForm} snippets={snippets} maxKeyLength={maxKeyLength} />
        </Modal>
      )}
    </div>
  )
}
