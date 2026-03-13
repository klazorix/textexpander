import { Routes, Route } from 'react-router-dom'
import Sidebar from './components/Sidebar'
import Dashboard from './pages/Dashboard'
import Snippets from './pages/Snippets'
import Triggers from './pages/Triggers'
import Variables from './pages/Variables'
import Hotkeys from './pages/Hotkeys'
import Settings from './pages/Settings'
import Credits from './pages/Credits'

export default function App() {
  return (
    <div className="flex h-screen bg-gray-950 text-white">
      <Sidebar />
      <main className="flex-1 overflow-y-auto p-8">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/snippets" element={<Snippets />} />
          <Route path="/triggers" element={<Triggers />} />
          <Route path="/variables" element={<Variables />} />
          <Route path="/hotkeys" element={<Hotkeys />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="/credits" element={<Credits />} />
        </Routes>
      </main>
    </div>
  )
}