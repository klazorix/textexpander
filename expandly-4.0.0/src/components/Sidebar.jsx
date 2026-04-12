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

const links = [
    { to: '/', icon: LayoutDashboard, label: 'Dashboard' },
    { to: '/snippets', icon: FileText, label: 'Snippets' },
    { to: '/triggers', icon: Zap, label: 'Triggers' },
    { to: '/variables', icon: Variable, label: 'Variables' },
    { to: '/hotkeys', icon: Keyboard, label: 'Hotkeys' },
]

const bottomLinks = [
    { to: '/settings', icon: Settings, label: 'Settings' },
    { to: '/about', icon: Heart, label: 'About Expandly' },
]

const linkClass = ({ isActive }) =>
    `flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${isActive
        ? 'bg-blue-600 text-white'
        : 'text-gray-400 hover:bg-gray-800 hover:text-white'
    }`

function isNewerVersion(latest, current) {
    const clean = v => v.replace(/^v/, '')
    const parse = v => {
        const str = clean(v)
        const match = str.match(/^(\d+\.\d+\.\d+)b(\d+)$/)
        if (match) return { parts: match[1].split('.').map(Number), pre: parseInt(match[2]) }
        return { parts: str.split('.').map(Number), pre: null }
    }
    const a = parse(latest)
    const b = parse(current)
    for (let i = 0; i < 3; i++) {
        if ((a.parts[i] ?? 0) > (b.parts[i] ?? 0)) return true
        if ((a.parts[i] ?? 0) < (b.parts[i] ?? 0)) return false
    }
    if (a.pre === null && b.pre !== null) return true
    if (a.pre !== null && b.pre === null) return false
    if (a.pre !== null && b.pre !== null) return a.pre > b.pre
    return false
}

function SidebarLink({ to, icon: Icon, label, hasUpdate = false }) {
    return (
        <NavLink key={to} to={to} end={to === '/'} className={linkClass}>
            <Icon size={18} />
            <span className="flex-1">{label}</span>
            {hasUpdate && <AlertCircle size={16} className="text-orange-400 shrink-0" />}
        </NavLink>
    )
}

export default function Sidebar() {
    const [appVersion, setAppVersion] = useState('...')
    const [hasUpdate, setHasUpdate] = useState(false)

    useEffect(() => {
        const { invoke } = window.__TAURI_INTERNALS__
        invoke('get_app_version').then(async version => {
            setAppVersion(version)
            try {
                const res = await fetch('https://api.github.com/repos/klazorix/Expandly/releases/latest')
                const candidate = await res.json()
                setHasUpdate(Boolean(candidate?.tag_name && isNewerVersion(candidate.tag_name, version)))
            } catch { }
        })
    }, [])

    return (
        <aside className="w-56 h-screen bg-gray-900 border-r border-gray-800 flex flex-col py-6 px-3 shrink-0">
            <div className="mb-8 px-3 flex items-center gap-3">
                <img src={logo} alt="Expandly" className="w-8 h-8 rounded-lg shrink-0" />
                <div>
                    <h1 className="text-sm font-bold text-white">Expandly {appVersion.split('.')[0]}</h1>
                    <p className="text-xs text-gray-500">v{appVersion}</p>
                </div>
            </div>

            <nav className="flex flex-col flex-1">
                <div className="flex flex-col gap-1">
                    {links.map(link => <SidebarLink key={link.to} {...link} />)}
                </div>

                <div className="flex flex-col gap-1 mt-auto pt-4 border-t border-gray-800">
                    {bottomLinks.map(link => (
                        <SidebarLink key={link.to} {...link} hasUpdate={link.label === 'Settings' && hasUpdate} />
                    ))}
                </div>
            </nav>
        </aside>
    )
}
