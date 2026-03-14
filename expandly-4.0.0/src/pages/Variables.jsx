import { useEffect, useState } from 'react'
import { Plus, Pencil, Trash2, X, Check, Copy } from 'lucide-react'

const { invoke } = window.__TAURI_INTERNALS__

const BUILTIN = [
  { token: '{date}',     description: 'Today\'s date',              example: '13/03/2026' },
  { token: '{time}',     description: 'Current time',               example: '14:35' },
  { token: '{datetime}', description: 'Date and time combined',     example: '13/03/2026 14:35' },
  { token: '{day}',      description: 'Day of the week',            example: 'Friday' },
  { token: '{month}',    description: 'Month name',                 example: 'March' },
  { token: '{year}',     description: 'Current year',               example: '2026' },
  { token: '{clipboard}',description: 'Current clipboard contents', example: '(whatever you last copied)' }
]

function Modal({ title, initial, onSave, onClose }) {
  const [name, setName] = useState(initial?.name ?? '')
  const [value, setValue] = useState(initial?.value ?? '')

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
            <label className="text-xs text-gray-400 mb-1 block">Variable Name</label>
            <div className="flex items-center gap-0">
              <span className="bg-gray-700 border border-r-0 border-gray-600 rounded-l-lg px-3 py-2 text-gray-400 text-sm font-mono">{`{`}</span>
              <input
                className="flex-1 bg-gray-800 border border-gray-700 px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors font-mono"
                placeholder="my_variable"
                value={name}
                onChange={e => setName(e.target.value.replace(/[^a-z0-9_]/gi, '_').toLowerCase())}
              />
              <span className="bg-gray-700 border border-l-0 border-gray-600 rounded-r-lg px-3 py-2 text-gray-400 text-sm font-mono">{`}`}</span>
            </div>
          </div>
          <div>
            <label className="text-xs text-gray-400 mb-1 block">Value</label>
            <input
              className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white text-sm focus:outline-none focus:border-blue-500 transition-colors"
              placeholder="e.g. Acme Corp"
              value={value}
              onChange={e => setValue(e.target.value)}
            />
          </div>
        </div>

        <div className="flex justify-end gap-2 mt-6">
          <button onClick={onClose} className="px-4 py-2 rounded-lg text-sm text-gray-400 hover:text-white hover:bg-gray-800 transition-colors">
            Cancel
          </button>
          <button
            onClick={() => onSave({ name, value })}
            disabled={!name.trim() || !value.trim()}
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

export default function Variables() {
  const [customs, setCustoms] = useState([])
  const [showAdd, setShowAdd] = useState(false)
  const [editing, setEditing] = useState(null)
  const [copied, setCopied] = useState(null)

  const load = () =>
    invoke('get_config').then(c => setCustoms(c.custom_variables))

  useEffect(() => { load() }, [])

  const copy = (token) => {
    navigator.clipboard.writeText(token)
    setCopied(token)
    setTimeout(() => setCopied(null), 1500)
  }

  const handleAdd = async ({ name, value }) => {
    await invoke('create_custom_variable', { name, value })
    setShowAdd(false)
    load()
  }

  const handleEdit = async ({ name, value }) => {
    await invoke('update_custom_variable', { id: editing.id, name, value })
    setEditing(null)
    load()
  }

  const handleDelete = async (id) => {
    await invoke('delete_custom_variable', { id })
    load()
  }

  return (
    <div className="max-w-4xl mx-auto">

      <div className="mb-10">
        <h1 className="text-3xl font-bold text-white">Variables</h1>
        <p className="text-gray-400 mt-1">Use these tokens inside any snippet to insert dynamic content</p>
      </div>

      {/* Built-in */}
      <h2 className="text-xs font-semibold text-gray-500 uppercase tracking-widest mb-3">Built-in</h2>
      <div className="flex flex-col gap-2 mb-10">
        {BUILTIN.map(v => (
          <div key={v.token} className="bg-gray-900 border border-gray-800 rounded-xl px-5 py-3 flex items-center justify-between gap-4 group hover:border-gray-700 transition-colors">
            <div className="flex items-center gap-4 min-w-0">
              <span className="font-mono text-blue-400 bg-blue-500/10 px-2 py-0.5 rounded text-sm shrink-0">{v.token}</span>
              <span className="text-gray-400 text-sm">{v.description}</span>
              <span className="text-gray-600 text-xs hidden md:block">e.g. {v.example}</span>
            </div>
            <button
              onClick={() => copy(v.token)}
              className="p-2 rounded-lg text-gray-600 hover:text-white hover:bg-gray-800 transition-colors opacity-0 group-hover:opacity-100 shrink-0"
            >
              {copied === v.token ? <Check size={14} className="text-green-400" /> : <Copy size={14} />}
            </button>
          </div>
        ))}
      </div>

      {/* Custom */}
      <div className="flex items-center justify-between mb-3">
        <h2 className="text-xs font-semibold text-gray-500 uppercase tracking-widest">Custom</h2>
        <button
          onClick={() => setShowAdd(true)}
          className="flex items-center gap-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-medium px-4 py-2 rounded-xl transition-colors"
        >
          <Plus size={16} />
          New Variable
        </button>
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
              <button onClick={() => setEditing(v)} className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-gray-800 transition-colors">
                <Pencil size={15} />
              </button>
              <button onClick={() => handleDelete(v.id)} className="p-2 rounded-lg text-gray-400 hover:text-red-400 hover:bg-gray-800 transition-colors">
                <Trash2 size={15} />
              </button>
            </div>
          </div>
        ))}
      </div>

      {showAdd && <Modal title="New Variable" onSave={handleAdd} onClose={() => setShowAdd(false)} />}
      {editing && <Modal title="Edit Variable" initial={editing} onSave={handleEdit} onClose={() => setEditing(null)} />}
    </div>
  )
}