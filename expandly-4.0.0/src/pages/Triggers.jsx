import { useEffect, useState } from 'react'
import { Plus, Pencil, Trash2, X, Check, Zap } from 'lucide-react'

const { invoke } = window.__TAURI_INTERNALS__

function Modal({ title, initial, snippets, onSave, onClose }) {
  const [key, setKey] = useState(initial?.key ?? '')
  const [expansionId, setExpansionId] = useState(initial?.expansion_id ?? '')
  const [wordBoundary, setWordBoundary] = useState(initial?.word_boundary ?? true)

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
      <div className="bg-gray-900 border border-gray-700 rounded-2xl p-6 w-full max-w-lg shadow-2xl">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-semibold text-white">{title}</h2>
          <button onClick={onClose} className="text-gray-500 hover:text-white transition-colors">
            <X size={20} />
          </button>
        </div>

        <div className="flex flex-col gap-4">
          <div>
            <label className="text-xs text-gray-400 mb-1 block">Trigger Text</label>
            <input
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors font-mono"
              placeholder="e.g. /hi"
              value={key}
              maxLength={16}
              onChange={e => setKey(e.target.value)}
            />
            <p className={`text-xs mt-1 text-right ${key.length >= 16 ? 'text-red-400' : 'text-gray-600'}`}>
              {key.length}/16
            </p>
          </div>

          <div>
            <label className="text-xs text-gray-400 mb-1 block">Expands To</label>
            <select
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors"
              value={expansionId}
              onChange={e => setExpansionId(e.target.value)}
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
              onClick={() => setWordBoundary(v => !v)}
              className={`w-11 h-6 rounded-full transition-colors relative ${wordBoundary ? 'bg-blue-600' : 'bg-gray-600'}`}
            >
              <span className={`absolute top-0.5 w-5 h-5 bg-white rounded-full shadow transition-all ${wordBoundary ? 'left-5' : 'left-0.5'}`} />
            </button>
          </div>
        </div>

        <div className="flex justify-end gap-2 mt-6">
          <button onClick={onClose} className="px-4 py-2 rounded-lg text-sm text-gray-400 hover:text-white hover:bg-gray-800 transition-colors">
            Cancel
          </button>
          <button
            onClick={() => onSave({ key, expansion_id: expansionId, word_boundary: wordBoundary })}
            disabled={!key.trim() || !expansionId}
            className="px-4 py-2 rounded-lg text-sm bg-blue-600 hover:bg-blue-500 text-white font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed flex items-center gap-2"
          >
            <Check size={15} />
            Save
          </button>
        </div>
      </div>
    </div>
  )
}

export default function Triggers() {
  const [triggers, setTriggers] = useState([])
  const [snippets, setSnippets] = useState([])
  const [showAdd, setShowAdd] = useState(false)
  const [editing, setEditing] = useState(null)

  const load = () =>
    invoke('get_config').then(c => {
      setTriggers(c.triggers)
      setSnippets(Object.values(c.expansions))
    })

  useEffect(() => { load() }, [])

  const snippetName = id => snippets.find(s => s.id === id)?.name ?? 'Unknown'

  const handleAdd = async (data) => {
    await invoke('create_trigger', { key: data.key, expansionId: data.expansion_id, wordBoundary: data.word_boundary })
    setShowAdd(false)
    load()
  }

  const handleEdit = async (data) => {
    await invoke('update_trigger', { id: editing.id, key: data.key, expansionId: data.expansion_id, wordBoundary: data.word_boundary })
    setEditing(null)
    load()
  }

  const handleDelete = async (id) => {
    await invoke('delete_trigger', { id })
    load()
  }

  return (
    <div className="max-w-4xl mx-auto">
      <div className="flex items-center justify-between mb-10">
        <div>
          <h1 className="text-3xl font-bold text-white">Triggers</h1>
          <p className="text-gray-400 mt-1">{triggers.length} trigger{triggers.length !== 1 ? 's' : ''} configured</p>
        </div>
        <button
          onClick={() => setShowAdd(true)}
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
                <span className="text-xs text-gray-600 bg-gray-800 px-2 py-0.5 rounded-full shrink-0">word boundary</span>
              )}
            </div>
            <div className="flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
              <button onClick={() => setEditing(t)} className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-gray-800 transition-colors">
                <Pencil size={15} />
              </button>
              <button onClick={() => handleDelete(t.id)} className="p-2 rounded-lg text-gray-400 hover:text-red-400 hover:bg-gray-800 transition-colors">
                <Trash2 size={15} />
              </button>
            </div>
          </div>
        ))}
      </div>

      {showAdd && <Modal title="New Trigger" snippets={snippets} onSave={handleAdd} onClose={() => setShowAdd(false)} />}
      {editing && <Modal title="Edit Trigger" initial={editing} snippets={snippets} onSave={handleEdit} onClose={() => setEditing(null)} />}
    </div>
  )
}