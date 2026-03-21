import { useState } from 'react'
import { Plus, Pencil, Trash2, Check, Copy, BookOpen, X } from 'lucide-react'
import { useInvoke } from '../hooks/useInvoke'
import { useConfig } from '../hooks/useConfig'
import Modal from '../components/Modal'

const BUILTIN = [
  { token: '{date}', description: "Today's date", example: '13/03/2026' },
  { token: '{time}', description: 'Current time', example: '14:35' },
  { token: '{datetime}', description: 'Date and time combined', example: '17/03/2026 14:35' },
  { token: '{day}', description: 'Day of the week', example: 'Friday' },
  { token: '{month}', description: 'Month name', example: 'March' },
  { token: '{year}', description: 'Current year', example: '2026' },
  { token: '{hour}', description: 'Current hour', example: '14' },
  { token: '{minute}', description: 'Current minute', example: '35' },
  { token: '{yesterday}', description: "Yesterday's date", example: '16/03/2026' },
  { token: '{tomorrow}', description: "Tomorrow's date", example: '18/03/2026' },
  { token: '{greeting}', description: 'Time-based greeting', example: 'Good afternoon' },
  { token: '{clipboard}', description: 'Current clipboard contents', example: '(last copied text)' },
]

function VariableForm({ form, onChange }) {
  return (
    <>
      <div>
        <label className="text-xs text-gray-400 mb-1 block">Variable Name</label>
        <div className="flex items-center gap-0">
          <span className="bg-gray-700 border border-r-0 border-gray-600 rounded-l-lg px-3 py-2 text-gray-400 text-sm font-mono">{`{`}</span>
          <input
            className="flex-1 bg-gray-800 border border-gray-700 px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors font-mono"
            placeholder="my_variable"
            value={form.name}
            onChange={e => onChange({ ...form, name: e.target.value.replace(/[^a-z0-9_]/gi, '_').toLowerCase() })}
          />
          <span className="bg-gray-700 border border-l-0 border-gray-600 rounded-r-lg px-3 py-2 text-gray-400 text-sm font-mono">{`}`}</span>
        </div>
      </div>
      <div>
        <label className="text-xs text-gray-400 mb-1 block">Value</label>
        <input
          className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors"
          placeholder="e.g. Acme Corp"
          value={form.value}
          onChange={e => onChange({ ...form, value: e.target.value })}
        />
      </div>
    </>
  )
}

function BuiltinModal({ onClose }) {
  const [copied, setCopied] = useState(null)

  const copy = (token) => {
    navigator.clipboard.writeText(token)
    setCopied(token)
    setTimeout(() => setCopied(null), 1500)
  }

  return (
    <div className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50 p-6">
      <div className="bg-gray-900 border border-gray-700 rounded-2xl w-full max-w-2xl max-h-[80vh] flex flex-col shadow-2xl">
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-800 shrink-0">
          <div>
            <h2 className="text-white font-semibold">Built-in Variables</h2>
            <p className="text-gray-500 text-xs mt-0.5">Click any placeholder to copy it</p>
          </div>
          <button onClick={onClose} className="text-gray-500 hover:text-white transition-colors">
            <X size={20} />
          </button>
        </div>
        <div className="overflow-y-auto px-6 py-4 flex-1 flex flex-col gap-2">
          {BUILTIN.map(v => (
            <div
              key={v.token}
              className="bg-gray-800 border border-gray-700 rounded-xl px-4 py-3 flex items-center justify-between gap-4 hover:border-gray-600 transition-colors"
            >
              <div className="flex items-center gap-3 min-w-0">
                <span className="font-mono text-blue-400 bg-blue-500/10 px-2 py-0.5 rounded text-sm shrink-0">{v.token}</span>
                <div className="min-w-0">
                  <p className="text-gray-300 text-xs">{v.description}</p>
                  <p className="text-gray-600 text-xs mt-0.5">e.g. {v.example}</p>
                </div>
              </div>
              <button
                onClick={() => copy(v.token)}
                className="p-2 rounded-lg text-gray-500 hover:text-white hover:bg-gray-700 transition-colors shrink-0"
              >
                {copied === v.token ? <Check size={14} className="text-green-400" /> : <Copy size={14} />}
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

export default function Variables() {
  const invoke = useInvoke()
  const { config, reload } = useConfig()
  const customs = config?.custom_variables ?? []

  const blank = { name: '', value: '' }
  const [showAdd, setShowAdd] = useState(false)
  const [editing, setEditing] = useState(null)
  const [addForm, setAddForm] = useState(blank)
  const [editForm, setEditForm] = useState(blank)
  const [copied, setCopied] = useState(null)
  const [showBuiltin, setShowBuiltin] = useState(false)

  const copy = (token) => {
    navigator.clipboard.writeText(token)
    setCopied(token)
    setTimeout(() => setCopied(null), 1500)
  }

  const handleAdd = async () => {
    await invoke('create_custom_variable', { name: addForm.name, value: addForm.value })
    setShowAdd(false)
    setAddForm(blank)
    reload()
  }

  const handleEdit = async () => {
    await invoke('update_custom_variable', { id: editing.id, name: editForm.name, value: editForm.value })
    setEditing(null)
    reload()
  }

  const handleDelete = async (id) => {
    await invoke('delete_custom_variable', { id })
    reload()
  }

  const openEdit = (v) => {
    setEditForm({ name: v.name, value: v.value })
    setEditing(v)
  }

  return (
    <div className="max-w-4xl mx-auto">

      <div className="flex items-center justify-between gap-6 mb-10">
        <div className="min-w-0">
          <h1 className="text-3xl font-bold text-white">Variables</h1>
          <p className="text-gray-400 mt-1">Use placeholders inside any snippet to insert dynamic content</p>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <button
            onClick={() => setShowBuiltin(true)}
            className="flex items-center gap-2 px-4 py-2 rounded-xl bg-gray-900 border border-gray-800 hover:border-gray-700 text-gray-300 hover:text-white text-sm transition-colors"
          >
            <BookOpen size={15} />
            Built-in Variables
          </button>
          <button
            onClick={() => { setAddForm(blank); setShowAdd(true) }}
            className="flex items-center gap-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium px-4 py-2 rounded-xl transition-colors"
          >
            <Plus size={16} />
            New Variable
          </button>
        </div>
      </div>

      {customs.length === 0 && (
        <div className="bg-gray-900 border border-gray-800 border-dashed rounded-2xl py-12 text-center">
          <p className="text-gray-600 text-sm">No custom variables yet</p>
        </div>
      )}

      <div className="flex flex-col gap-2">
        {customs.map(v => (
          <div key={v.id} className="bg-gray-900 border border-gray-800 rounded-xl px-5 py-3 flex items-center justify-between gap-4 group hover:border-gray-700 transition-colors">
            <div className="flex items-center gap-4 min-w-0">
              <span className="font-mono text-violet-400 bg-violet-500/10 px-2 py-0.5 rounded text-sm shrink-0">{`{${v.name}}`}</span>
              <span className="text-gray-300 text-sm truncate">{v.value}</span>
            </div>
            <div className="flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
              <button onClick={() => copy(`{${v.name}}`)} className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-gray-800 transition-colors">
                {copied === `{${v.name}}` ? <Check size={14} className="text-green-400" /> : <Copy size={14} />}
              </button>
              <button onClick={() => openEdit(v)} className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-gray-800 transition-colors">
                <Pencil size={15} />
              </button>
              <button onClick={() => handleDelete(v.id)} className="p-2 rounded-lg text-gray-400 hover:text-red-400 hover:bg-gray-800 transition-colors">
                <Trash2 size={15} />
              </button>
            </div>
          </div>
        ))}
      </div>

      {showAdd && (
        <Modal
          title="New Variable"
          onClose={() => setShowAdd(false)}
          onSave={handleAdd}
          disabled={!addForm.name.trim() || !addForm.value.trim()}
        >
          <VariableForm form={addForm} onChange={setAddForm} />
        </Modal>
      )}

      {editing && (
        <Modal
          title="Edit Variable"
          onClose={() => setEditing(null)}
          onSave={handleEdit}
          disabled={!editForm.name.trim() || !editForm.value.trim()}
        >
          <VariableForm form={editForm} onChange={setEditForm} />
        </Modal>
      )}

      {showBuiltin && <BuiltinModal onClose={() => setShowBuiltin(false)} />}
    </div>
  )
}
