import { Routes, Route } from 'react-router-dom'
import Sidebar from './components/Sidebar'
import Dashboard from './pages/Dashboard'
import Snippets from './pages/Snippets'
import Triggers from './pages/Triggers'
import Variables from './pages/Variables'
import Hotkeys from './pages/Hotkeys'
import Settings from './pages/Settings'
import About from './pages/About'

const routes = [
  ['/', Dashboard],
  ['/snippets', Snippets],
  ['/triggers', Triggers],
  ['/variables', Variables],
  ['/hotkeys', Hotkeys],
  ['/settings', Settings],
  ['/about', About],
]

export default function App() {
  return (
    <div className="flex h-screen bg-gray-950 text-white">
      <Sidebar />
      <main className="flex-1 overflow-y-auto p-8">
        <Routes>
          {routes.map(([path, Component]) => (
            <Route key={path} path={path} element={<Component />} />
          ))}
        </Routes>
      </main>
    </div>
  )
}