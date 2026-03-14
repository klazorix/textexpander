import { execSync, spawn } from 'child_process'
import { existsSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = join(__dirname, '..')

const RESET  = '\x1b[0m'
const BOLD   = '\x1b[1m'
const GREEN  = '\x1b[32m'
const BLUE   = '\x1b[34m'
const YELLOW = '\x1b[33m'
const RED    = '\x1b[31m'
const CYAN   = '\x1b[36m'
const DIM    = '\x1b[2m'

function log(color, symbol, msg) {
  console.log(`${color}${BOLD}${symbol}${RESET} ${msg}`)
}

function step(msg)    { log(BLUE,   '→', msg) }
function success(msg) { log(GREEN,  '✔', msg) }
function warn(msg)    { log(YELLOW, '!', msg) }
function error(msg)   { log(RED,    '✘', msg) }
function info(msg)    { log(CYAN,   'i', msg) }
function dim(msg)     { console.log(`${DIM}  ${msg}${RESET}`) }

function run(cmd, options = {}) {
  return new Promise((resolve, reject) => {
    const [bin, ...args] = cmd.split(' ')
    const proc = spawn(bin, args, {
      cwd: options.cwd ?? root,
      stdio: 'inherit',
      shell: true,
    })
    proc.on('close', code => {
      if (code === 0) resolve()
      else reject(new Error(`Command failed: ${cmd} (exit ${code})`))
    })
  })
}

function check(cmd) {
  try { execSync(`${cmd} --version`, { stdio: 'ignore' }); return true }
  catch { return false }
}

async function main() {
  console.log()
  console.log(`${BOLD}${BLUE}╔════════════════════════════════════════╗${RESET}`)
  console.log(`${BOLD}${BLUE}║         Expandly v4.0.0 — Setup        ║${RESET}`)
  console.log(`${BOLD}${BLUE}╚════════════════════════════════════════╝${RESET}`)
  console.log()

  // ── 1. Check prerequisites ───────────────────────────────────────────────
  step('Checking prerequisites...')

  const hasNode  = check('node')
  const hasCargo = check('cargo')
  const hasTauri = check('cargo tauri')

  if (!hasNode)  { error('Node.js not found. Install from https://nodejs.org'); process.exit(1) }
  if (!hasCargo) { error('Rust/Cargo not found. Install from https://rustup.rs'); process.exit(1) }

  success(`Node.js   ${execSync('node --version').toString().trim()}`)
  success(`Cargo     ${execSync('cargo --version').toString().trim()}`)

  if (!hasTauri) {
    warn('Tauri CLI not found — installing now...')
    await run('cargo install tauri-cli --version "^2" --locked')
    success('Tauri CLI installed')
  } else {
    success('Tauri CLI found')
  }

  console.log()

  // ── 2. Install Node dependencies ─────────────────────────────────────────
  const nodeModulesExists = existsSync(join(root, 'node_modules'))

  if (nodeModulesExists) {
    step('node_modules already exists — running npm install to sync...')
  } else {
    step('Installing Node dependencies...')
  }

  await run('npm install')
  success('Node dependencies ready')
  console.log()

  // ── 3. Tailwind check ────────────────────────────────────────────────────
  step('Checking Tailwind CSS...')
  const hasTailwind = existsSync(join(root, 'node_modules', 'tailwindcss'))
  if (!hasTailwind) {
    step('Installing Tailwind CSS...')
    await run('npm install -D tailwindcss@3 postcss autoprefixer @tailwindcss/postcss')
  }
  success('Tailwind CSS ready')
  console.log()

  // ── 4. Fetch Rust crates ─────────────────────────────────────────────────
  step('Fetching Rust crates (this may take a moment on first run)...')
  await run('cargo fetch', { cwd: join(root, 'src-tauri') })
  success('Rust crates ready')
  console.log()

  // ── 5. Done ──────────────────────────────────────────────────────────────
  console.log(`${GREEN}${BOLD}╔════════════════════════════════════════╗${RESET}`)
  console.log(`${GREEN}${BOLD}║   Setup complete!                      ║${RESET}`)
  console.log(`${GREEN}${BOLD}╚════════════════════════════════════════╝${RESET}`)
  console.log()
  info('To start the app in development mode:')
  dim('npm run tauri dev')
  console.log()
  info('To build for production:')
  dim('npm run tauri build')
  console.log()
}

main().catch(e => {
  error(e.message)
  process.exit(1)
})