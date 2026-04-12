import { useState } from 'react'
import { Plus, Pencil, Trash2, FileText } from 'lucide-react'
import { useInvoke } from '../hooks/useInvoke'
import { useConfig } from '../hooks/useConfig'
import Modal from '../components/Modal'

const blankForm = { name: '', text: '' }

function SnippetForm({ initial, onChange }) {
  return (
    <>
      <div>
        <label className="text-xs text-gray-400 mb-1 block">Snippet Name</label>
        <input
          className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors"
          placeholder="e.g. My Email Address"
          value={initial.name}
          onChange={e => onChange({ ...initial, name: e.target.value })}
        />
      </div>
      <div>
        <label className="text-xs text-gray-400 mb-1 block">Expansion Text</label>
        <textarea
          className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors resize-none"
          placeholder="e.g. hello@example.com"
          rows={5}
          value={initial.text}
          onChange={e => onChange({ ...initial, text: e.target.value })}
        />
      </div>
    </>
  )
}

export default function Snippets() {
  const invoke = useInvoke()
  const { config, reload } = useConfig()
  const expansions = config ? Object.values(config.expansions) : []

  const [showAdd, setShowAdd] = useState(false)
  const [editing, setEditing] = useState(null)
  const [addForm, setAddForm] = useState(blankForm)
  const [editForm, setEditForm] = useState(blankForm)

  const handleAdd = async () => {
    await invoke('create_expansion', { name: addForm.name, text: addForm.text })
    setShowAdd(false)
    setAddForm(blankForm)
    reload()
  }

  const handleEdit = async () => {
    await invoke('update_expansion', { id: editing.id, name: editForm.name, text: editForm.text })
    setEditing(null)
    reload()
  }

  const handleDelete = async (id) => {
    await invoke('delete_expansion', { id })
    reload()
  }

  const openEdit = (exp) => {
    setEditForm({ name: exp.name, text: exp.text })
    setEditing(exp)
  }

  return (
    <div className="max-w-4xl mx-auto">

      <div className="flex items-center justify-between mb-10">
        <div>
          <h1 className="text-3xl font-bold text-white">Snippets</h1>
          <p className="text-gray-400 mt-1">{expansions.length} snippet{expansions.length !== 1 ? 's' : ''} saved</p>
        </div>
        <button
          onClick={() => { setAddForm(blankForm); setShowAdd(true) }}
          className="flex items-center gap-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium px-4 py-2 rounded-xl transition-colors"
        >
          <Plus size={16} />
          New Snippet
        </button>
      </div>

      {expansions.length === 0 && (
        <div className="flex flex-col items-center justify-center py-24 text-center">
          <div className="bg-gray-900 border border-gray-800 rounded-2xl p-5 mb-4">
            <FileText size={28} className="text-gray-600" />
          </div>
          <p className="text-gray-400 font-medium">No snippets yet</p>
          <p className="text-gray-600 text-sm mt-1">Click "New Snippet" to create your first one</p>
        </div>
      )}

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
              <button onClick={() => openEdit(exp)} className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-gray-800 transition-colors">
                <Pencil size={15} />
              </button>
              <button onClick={() => handleDelete(exp.id)} className="p-2 rounded-lg text-gray-400 hover:text-red-400 hover:bg-gray-800 transition-colors">
                <Trash2 size={15} />
              </button>
            </div>
          </div>
        ))}
      </div>

      {showAdd && (
        <Modal
          title="New Snippet"
          onClose={() => setShowAdd(false)}
          onSave={handleAdd}
          disabled={!addForm.name.trim() || !addForm.text.trim()}
        >
          <SnippetForm initial={addForm} onChange={setAddForm} />
        </Modal>
      )}

      {editing && (
        <Modal
          title="Edit Snippet"
          onClose={() => setEditing(null)}
          onSave={handleEdit}
          disabled={!editForm.name.trim() || !editForm.text.trim()}
        >
          <SnippetForm initial={editForm} onChange={setEditForm} />
        </Modal>
      )}
    </div>
  )
}
