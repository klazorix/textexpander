import { useEffect, useState } from 'react'
import { FileText, Zap, Keyboard, Variable, Trophy, Calendar, CalendarDays, Infinity } from 'lucide-react'
import { useConfig } from '../hooks/useConfig'
import { useInvoke } from '../hooks/useInvoke'

export default function Dashboard() {
  const { config } = useConfig(5000)
  const invoke = useInvoke()
  const [appVersion, setAppVersion] = useState('...')

  useEffect(() => {
    invoke('get_app_version').then(setAppVersion)
  }, [])

  const snippetCount = config ? Object.keys(config.expansions).length : 0
  const triggerCount = config ? config.triggers.length : 0
  const hotkeyCount = config ? config.hotkeys.length : 0
  const variableCount = config ? config.custom_variables.length : 0
  const totalExpansions = config?.stats.total_expansions ?? 0

  const todayKey = (() => {
    const d = new Date()
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
  })()

  const todayExpansions = config?.stats.expansions_per_day?.[todayKey] ?? 0

  const thisWeek = (() => {
    if (!config?.stats.expansions_per_day) return 0
    let total = 0
    for (let i = 0; i < 7; i++) {
      const d = new Date()
      d.setDate(d.getDate() - i)
      const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
      total += config.stats.expansions_per_day[key] ?? 0
    }
    return total
  })()

  const leaderboard = (() => {
    if (!config?.stats.expansion_counts) return []
    return Object.entries(config.stats.expansion_counts)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5)
      .map(([id, count]) => ({
        name: config.expansions[id]?.name ?? 'Deleted Snippet',
        count,
      }))
  })()

  const countCards = [
    { label: 'Snippets', value: snippetCount, icon: FileText, color: 'from-blue-600 to-blue-400' },
    { label: 'Triggers', value: triggerCount, icon: Zap, color: 'from-violet-600 to-violet-400' },
    { label: 'Hotkeys', value: hotkeyCount, icon: Keyboard, color: 'from-pink-600 to-pink-400' },
    { label: 'Variables', value: variableCount, icon: Variable, color: 'from-cyan-600 to-cyan-400' },
  ]

  return (
    <div className="max-w-5xl mx-auto">

      <div className="mb-8">
        <h1 className="text-3xl font-bold text-white">Dashboard</h1>
        <p className="text-gray-400 mt-1">Welcome back. Here's what's happening.</p>
      </div>

      <div className="bg-gray-900 border border-gray-800 rounded-2xl px-6 py-4 mb-6 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className={`w-2.5 h-2.5 rounded-full ${config?.enabled ? 'bg-emerald-400' : 'bg-red-400'}`} />
          <div>
            <div className="flex items-center gap-2">
              <p className="text-white font-semibold text-sm">Expandly {appVersion.split('.')[0]}</p>
              <span className="text-gray-700">|</span>
              <p className="text-gray-400 text-sm">Engine</p>
            </div>
            <p className="text-gray-500 text-xs">v{appVersion}</p>
          </div>
        </div>
        <span className={`text-xs font-medium px-3 py-1 rounded-full ${config?.enabled
          ? 'bg-emerald-500/10 text-emerald-400'
          : 'bg-red-500/10 text-red-400'
          }`}>
          {config?.enabled ? 'Active' : 'Disabled'}
        </span>
      </div>

      <div className="grid grid-cols-4 gap-4 mb-6">
        {countCards.map(({ label, value, icon: Icon, color }) => (
          <div
            key={label}
            className="bg-gray-900 border border-gray-800 rounded-2xl p-5 flex items-center gap-4 hover:border-gray-700 transition-colors"
          >
            <div className={`bg-gradient-to-br ${color} p-3 rounded-xl shrink-0`}>
              <Icon size={18} className="text-white" />
            </div>
            <div className="min-w-0">
              <p className="text-xl font-bold text-white">{value}</p>
              <p className="text-xs text-gray-400 mt-0.5">{label}</p>
            </div>
          </div>
        ))}
      </div>

      {config && !config.track_stats ? (
        <div className="bg-gray-900 border border-gray-800 border-dashed rounded-2xl p-10 flex flex-col items-center justify-center text-center">
          <p className="text-gray-400 font-medium mb-1">Usage statistics disabled</p>
          <p className="text-gray-600 text-sm">Enable statistics tracking in <span className="text-gray-500">Settings → Data</span> to see your expansion history and leaderboard.</p>
        </div>
      ) : (
        <div className="grid grid-cols-5 gap-4">

          <div className="col-span-3 bg-gray-900 border border-gray-800 rounded-2xl p-5">
            <div className="flex items-center gap-2 mb-5">
              <Trophy size={16} className="text-amber-400" />
              <h2 className="text-sm font-semibold text-white">Most Used Snippets</h2>
            </div>
            {leaderboard.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-8 text-center">
                <p className="text-gray-600 text-sm">No expansions recorded yet</p>
                <p className="text-gray-700 text-xs mt-1">Use a trigger or hotkey to get started</p>
              </div>
            ) : (
              <div className="flex flex-col gap-4">
                {leaderboard.map((item, i) => (
                  <div key={i} className="flex items-center gap-3">
                    <span className={`text-xs font-bold w-5 shrink-0 ${i === 0 ? 'text-amber-400' : i === 1 ? 'text-gray-300' : i === 2 ? 'text-orange-600' : 'text-gray-600'}`}>
                      #{i + 1}
                    </span>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center justify-between mb-1.5">
                        <span className="text-white text-xs font-medium truncate">{item.name}</span>
                        <span className="text-gray-500 text-xs ml-2 shrink-0">{item.count.toLocaleString()}</span>
                      </div>
                      <div className="w-full bg-gray-800 rounded-full h-1">
                        <div
                          className={`h-1 rounded-full transition-all ${i === 0 ? 'bg-amber-400' : i === 1 ? 'bg-gray-300' : i === 2 ? 'bg-orange-600' : 'bg-gray-700'}`}
                          style={{ width: `${(item.count / (leaderboard[0]?.count || 1)) * 100}%` }}
                        />
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          <div className="col-span-2 flex flex-col gap-4">
            {[
              { label: 'Today', value: todayExpansions, icon: Calendar, color: 'from-orange-600 to-orange-400' },
              { label: 'This Week', value: thisWeek, icon: CalendarDays, color: 'from-teal-600 to-teal-400' },
              { label: 'All Time', value: totalExpansions, icon: Infinity, color: 'from-emerald-600 to-emerald-400' },
            ].map(({ label, value, icon: Icon, color }) => (
              <div key={label} className="bg-gray-900 border border-gray-800 rounded-2xl p-5 flex items-center gap-4 flex-1 hover:border-gray-700 transition-colors">
                <div className={`bg-gradient-to-br ${color} p-3 rounded-xl shrink-0`}>
                  <Icon size={18} className="text-white" />
                </div>
                <div>
                  <p className="text-2xl font-bold text-white">{value.toLocaleString()}</p>
                  <p className="text-xs text-gray-400 mt-0.5">{label}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
