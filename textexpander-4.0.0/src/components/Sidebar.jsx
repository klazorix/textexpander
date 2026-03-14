import { NavLink } from 'react-router-dom'
import { useEffect, useState } from 'react'
import {
  LayoutDashboard,
  FileText,
  Zap,
  Variable,
  Keyboard,
  Heart,
  RefreshCw,
  AlertCircle,
  Settings
} from 'lucide-react'

const CURRENT_VERSION = '4.0.0'
const GITHUB_REPO = 'klazorix/expandly'

const links = [
  { to: '/',          icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/snippets',  icon: FileText,        label: 'Snippets'  },
  { to: '/triggers',  icon: Zap,             label: 'Triggers'  },
  { to: '/variables', icon: Variable,        label: 'Variables' },
  { to: '/hotkeys',   icon: Keyboard,        label: 'Hotkeys'   },
  { to: '/settings',   icon: Settings,        label: 'Settings'   },
  { to: '/credits',   icon: Heart,           label: 'Credits'   },
]

function newerVersion(latest, current) {
  const a = latest.replace(/^v/, '').split('.').map(Number)
  const b = current.replace(/^v/, '').split('.').map(Number)
  for (let i = 0; i < 3; i++) {
    if ((a[i] ?? 0) > (b[i] ?? 0)) return true
    if ((a[i] ?? 0) < (b[i] ?? 0)) return false
  }
  return false
}

export default function Sidebar() {
  const [hasUpdate, setHasUpdate] = useState(false)

  useEffect(() => {
    fetch(`https://api.github.com/repos/${GITHUB_REPO}/releases/latest`)
      .then(r => r.json())
      .then(data => {
        if (data.tag_name && newerVersion(data.tag_name, CURRENT_VERSION)) {
          setHasUpdate(true)
        }
      })
      .catch(() => {})
  }, [])

  return (
    <aside className="w-56 h-screen bg-gray-900 border-r border-gray-800 flex flex-col py-6 px-3 shrink-0">
      <div className="mb-8 px-3">
        <h1 className="text-lg font-bold text-white">Expandly</h1>
        <p className="text-xs text-gray-500">v4.0.0</p>
      </div>

      <nav className="flex flex-col gap-1">
        {links.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            end={to === '/'}
            className={({ isActive }) =>
              `flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${
                isActive
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-400 hover:bg-gray-800 hover:text-white'
              }`
            }
          >
            <Icon size={18} />
            <span className="flex-1">{label}</span>
            {label === 'Updates' && hasUpdate && (
              <AlertCircle size={16} className="text-orange-400 shrink-0" />
            )}
          </NavLink>
        ))}
      </nav>
    </aside>
  )
}