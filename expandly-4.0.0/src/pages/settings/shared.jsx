import { Settings as SettingsIcon, Palette, Database, RefreshCw } from 'lucide-react'

export const tabs = [
  { id: 'system', label: 'System', icon: SettingsIcon },
  { id: 'customise', label: 'Customise', icon: Palette },
  { id: 'data', label: 'Data', icon: Database },
  { id: 'updates', label: 'Updates', icon: RefreshCw },
]

export function SettingRow({ label, description, children }) {
  return (
    <div className="flex items-center justify-between gap-6 py-4 border-b border-gray-800 last:border-0">
      <div className="min-w-0">
        <p className="text-white text-sm font-medium">{label}</p>
        {description && <p className="text-gray-500 text-xs mt-0.5">{description}</p>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  )
}

export function SectionLabel({ children }) {
  return (
    <p className="text-xs font-semibold text-gray-500 uppercase tracking-widest mb-3 mt-6 first:mt-0">
      {children}
    </p>
  )
}

export function Card({ children }) {
  return (
    <div className="bg-gray-900 border border-gray-800 rounded-2xl px-6 mb-4">
      {children}
    </div>
  )
}
