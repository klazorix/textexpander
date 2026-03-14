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
import logo from '../../src-tauri/icons/128x128.png'

const { invoke } = window.__TAURI_INTERNALS__

const GITHUB_REPO = 'klazorix/expandly'

const links = [
    { to: '/',          icon: LayoutDashboard, label: 'Dashboard'  },
    { to: '/snippets',  icon: FileText,        label: 'Snippets'   },
    { to: '/triggers',  icon: Zap,             label: 'Triggers'   },
    { to: '/variables', icon: Variable,        label: 'Variables'  },
    { to: '/hotkeys',   icon: Keyboard,        label: 'Hotkeys'    },
]

const bottomLinks = [
    { to: '/settings', icon: Settings, label: 'Settings'       },
    { to: '/about',    icon: Heart,    label: 'About Expandly' },
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
    const [appVersion, setAppVersion] = useState('...')
    const [hasUpdate, setHasUpdate] = useState(false)

    useEffect(() => {
        invoke('get_app_version').then(v => {
            setAppVersion(v)
            fetch(`https://api.github.com/repos/${GITHUB_REPO}/releases/latest`)
                .then(r => r.json())
                .then(data => {
                    if (data.tag_name && newerVersion(data.tag_name, v)) {
                        setHasUpdate(true)
                    }
                })
                .catch(() => {})
        })
    }, [])

    return (
        <aside className="w-56 h-screen bg-gray-900 border-r border-gray-800 flex flex-col py-6 px-3 shrink-0">
            <div className="mb-8 px-3 flex items-center gap-3">
                <img src={logo} alt="Expandly" className="w-8 h-8 rounded-lg shrink-0" />
                <div>
                    <h1 className="text-sm font-bold text-white">Expandly</h1>
                    <p className="text-xs text-gray-500">v{appVersion}</p>
                </div>
            </div>

            <nav className="flex flex-col flex-1">
                <div className="flex flex-col gap-1">
                    {links.map(({ to, icon: Icon, label }) => (
                        <NavLink
                            key={to}
                            to={to}
                            end={to === '/'}
                            className={({ isActive }) =>
                                `flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${isActive
                                    ? 'bg-blue-600 text-white'
                                    : 'text-gray-400 hover:bg-gray-800 hover:text-white'
                                }`
                            }
                        >
                            <Icon size={18} />
                            <span className="flex-1">{label}</span>
                        </NavLink>
                    ))}
                </div>

                <div className="flex flex-col gap-1 mt-auto pt-4 border-t border-gray-800">
                    {bottomLinks.map(({ to, icon: Icon, label }) => (
                        <NavLink
                            key={to}
                            to={to}
                            end={to === '/'}
                            className={({ isActive }) =>
                                `flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${isActive
                                    ? 'bg-blue-600 text-white'
                                    : 'text-gray-400 hover:bg-gray-800 hover:text-white'
                                }`
                            }
                        >
                            <Icon size={18} />
                            <span className="flex-1">{label}</span>
                            {label === 'Settings' && hasUpdate && (
                                <AlertCircle size={16} className="text-orange-400 shrink-0" />
                            )}
                        </NavLink>
                    ))}
                </div>
            </nav>
        </aside>
    )
}