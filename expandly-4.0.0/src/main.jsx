import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import App from './App'
import './index.css'

const { invoke } = window.__TAURI_INTERNALS__

async function init() {
  try {
    const config = await invoke('get_config')
    document.documentElement.setAttribute('data-theme', config.theme ?? 'starry-blue')
  } catch {
    document.documentElement.setAttribute('data-theme', 'starry-blue')
  }

  ReactDOM.createRoot(document.getElementById('root')).render(
    <React.StrictMode>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </React.StrictMode>
  )
}

init()