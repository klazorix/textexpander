import { useEffect, useState } from 'react'
import { FileText, Variable, Keyboard, TrendingUp, Zap } from 'lucide-react'

const { invoke } = window.__TAURI_INTERNALS__

export default function Dashboard() {
  const [config, setConfig] = useState(null)

  useEffect(() => {
    invoke('get_config').then(setConfig)
  }, [])

  const snippetCount = config ? Object.keys(config.expansions).length : 0
  const triggerCount = config ? config.triggers.length : 0
  const totalExpansions = config?.stats.total_expansions ?? 0
  const charsSaved = config?.stats.total_chars_saved ?? 0

  const stats = [
    {
      label: 'Snippets',
      value: snippetCount,
      icon: FileText,
      color: 'from-blue-600 to-blue-400',
    },
    {
      label: 'Triggers',
      value: triggerCount,
      icon: Keyboard,
      color: 'from-violet-600 to-violet-400',
    },
    {
      label: 'Total Expansions',
      value: totalExpansions,
      icon: TrendingUp,
      color: 'from-emerald-600 to-emerald-400',
    },
    {
      label: 'Chars Saved',
      value: charsSaved,
      icon: Zap,
      color: 'from-amber-600 to-amber-400',
    },
  ]

  return (
    <div className="max-w-4xl mx-auto">

      {/* Header */}
      <div className="mb-10">
        <h1 className="text-3xl font-bold text-white">Dashboard</h1>
        <p className="text-gray-400 mt-1">Welcome back. Here's what's happening.</p>
      </div>

      {/* Stat Cards */}
      <div className="grid grid-cols-2 gap-4 mb-10">
        {stats.map(({ label, value, icon: Icon, color }) => (
          <div
            key={label}
            className="bg-gray-900 border border-gray-800 rounded-2xl p-6 flex items-center gap-5"
          >
            <div className={`bg-gradient-to-br ${color} p-3 rounded-xl`}>
              <Icon size={22} className="text-white" />
            </div>
            <div>
              <p className="text-3xl font-bold text-white">{value.toLocaleString()}</p>
              <p className="text-sm text-gray-400 mt-0.5">{label}</p>
            </div>
          </div>
        ))}
      </div>

      {/* Status */}
      <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 mb-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-white">Text Expander</h2>
            <p className="text-sm text-gray-400 mt-0.5">Global expansion engine</p>
          </div>
          <div className={`flex items-center gap-2 px-4 py-2 rounded-full text-sm font-medium ${
            config?.enabled
              ? 'bg-emerald-500/10 text-emerald-400'
              : 'bg-red-500/10 text-red-400'
          }`}>
            <span className={`w-2 h-2 rounded-full ${
              config?.enabled ? 'bg-emerald-400' : 'bg-red-400'
            }`} />
            {config?.enabled ? 'Active' : 'Disabled'}
          </div>
        </div>
      </div>

      {/* Version */}
      <p className="text-xs text-gray-600 text-right">
        Text Expander v{config?.version ?? '4.0.0'}
      </p>

    </div>
  )
}