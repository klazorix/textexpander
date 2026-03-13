import { useEffect, useState } from 'react'
import { Plus, Pencil, Trash2, X, Check, FileText } from 'lucide-react'

const { invoke } = window.__TAURI_INTERNALS__

function Modal({ title, initial, onSave, onClose }) {
  const [name, setName] = useState(initial?.name ?? '')
  const [text, setText] = useState(initial?.text ?? '')

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
            <label className="text-xs text-gray-400 mb-1 block">Snippet Name</label>
            <input
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors"
              placeholder="e.g. My Email Address"
              value={name}
              onChange={e => setName(e.target.value)}
            />
          </div>
          <div>
            <label className="text-xs text-gray-400 mb-1 block">Expansion Text</label>
            <textarea
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors resize-none"
              placeholder="e.g. hello@example.com"
              rows={5}
              value={text}
              onChange={e => setText(e.target.value)}
            />
          </div>
        </div>

        <div className="flex justify-end gap-2 mt-6">
          <button
            onClick={onClose}
            className="px-4 py-2 rounded-lg text-sm text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={() => onSave({ name, text })}
            disabled={!name.trim() || !text.trim()}
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

export default function Snippets() {
  const [expansions, setExpansions] = useState([])
  const [showAdd, setShowAdd] = useState(false)
  const [editing, setEditing] = useState(null)

  const load = () =>
    invoke('get_config').then(c =>
      setExpansions(Object.values(c.expansions))
    )

  useEffect(() => { load() }, [])

  const handleAdd = async ({ name, text }) => {
    await invoke('create_expansion', { name, text })
    setShowAdd(false)
    load()
  }

  const handleEdit = async ({ name, text }) => {
    await invoke('update_expansion', { id: editing.id, name, text })
    setEditing(null)
    load()
  }

  const handleDelete = async (id) => {
    await invoke('delete_expansion', { id })
    load()
  }

  return (
    <div className="max-w-4xl mx-auto">

      {/* Header */}
      <div className="flex items-center justify-between mb-10">
        <div>
          <h1 className="text-3xl font-bold text-white">Snippets</h1>
          <p className="text-gray-400 mt-1">{expansions.length} snippet{expansions.length !== 1 ? 's' : ''} saved</p>
        </div>
        <button
          onClick={() => setShowAdd(true)}
          className="flex items-center gap-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium px-4 py-2 rounded-xl transition-colors"
        >
          <Plus size={16} />
          New Snippet
        </button>
      </div>

      {/* Empty state */}
      {expansions.length === 0 && (
        <div className="flex flex-col items-center justify-center py-24 text-center">
          <div className="bg-gray-900 border border-gray-800 rounded-2xl p-5 mb-4">
            <FileText size={28} className="text-gray-600" />
          </div>
          <p className="text-gray-400 font-medium">No snippets yet</p>
          <p className="text-gray-600 text-sm mt-1">Click "New Snippet" to create your first one</p>
        </div>
      )}

      {/* Snippet list */}
      <div className="flex flex-col gap-3">
        {expansions.map(exp => (
          <div
            key={exp.id}
            className="bg-gray-900 border border-gray-800 rounded-2xl px-5 py-4 flex items-start justify-between gap-4 group hover:border-gray-700 transition-colors"
          >
            <div className="min-w-0">
              <p className="text-white font-medium text-sm">{exp.name}</p>
              <p className="text-gray-500 text-sm mt-1 truncate">{exp.text}</p>
            </div>
            <div className="flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                onClick={() => setEditing(exp)}
                className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
              >
                <Pencil size={15} />
              </button>
              <button
                onClick={() => handleDelete(exp.id)}
                className="p-2 rounded-lg text-gray-400 hover:text-red-400 hover:bg-gray-800 transition-colors"
              >
                <Trash2 size={15} />
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Modals */}
      {showAdd && (
        <Modal title="New Snippet" onSave={handleAdd} onClose={() => setShowAdd(false)} />
      )}
      {editing && (
        <Modal title="Edit Snippet" initial={editing} onSave={handleEdit} onClose={() => setEditing(null)} />
      )}
    </div>
  )
}