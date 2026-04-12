import { useEffect, useState } from 'react'
import { tabs } from './settings/shared'
import SystemTab from './settings/SystemTab'
import CustomiseTab from './settings/CustomiseTab'
import DataTab from './settings/DataTab'
import UpdatesTab from './settings/UpdatesTab'

export default function Settings() {
  const [activeTab, setActiveTab] = useState('engine')

  useEffect(() => {
    try {
      const state = window.history.state?.usr
      if (state?.tab) setActiveTab(state.tab)
    } catch { }
  }, [])

  return (
    <div className="max-w-3xl mx-auto">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-white">Settings</h1>
        <p className="text-gray-400 mt-1">Configure Expandly to your preference</p>
      </div>

      <div className="flex gap-1 bg-gray-900 border border-gray-800 rounded-xl p-1 mb-8">
        {tabs.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setActiveTab(id)}
            className={`flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
              activeTab === id ? 'bg-blue-600 text-white' : 'text-gray-400 hover:text-white hover:bg-gray-800'
            }`}
          >
            <Icon size={15} />
            {label}
          </button>
        ))}
      </div>

      {activeTab === 'engine' && <SystemTab />}
      {activeTab === 'appearance' && <CustomiseTab />}
      {activeTab === 'data' && <DataTab />}
      {activeTab === 'updates' && <UpdatesTab />}
    </div>
  )
}
