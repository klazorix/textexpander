# Expandly — Full Project Knowledge Base
> Complete context for development. Read fully before responding to any request. Source files should be attached separately.

---

## 1. Project Identity

- **Name:** Expandly (formerly TextExpander)
- **Version:** 4.0.0 (currently in beta, approaching stable release)
- **Type:** Desktop text expander — watches system-wide keystrokes and replaces typed triggers or hotkeys with saved snippets
- **Stack:** Tauri v2, Rust, React (JSX), Vite, Tailwind CSS v4
- **GitHub:** https://github.com/klazorix/Expandly
- **App data (Windows):** `C:\Users\<user>\AppData\Roaming\Expandly\`
- **Project root:** `expandly/expandly-4.0.0/`

---

## 2. Directory Structure

```
expandly/
├── LICENSE
├── README.md
└── expandly-4.0.0/
    ├── index.html
    ├── package.json
    ├── tailwind.config.js
    ├── postcss.config.js
    ├── vite.config.js
    ├── public/
    │   └── splash.html              ← Loading screen (transparent, shown before main window)
    ├── src/
    │   ├── main.jsx                 ← App entry, loads theme, calls close_splash after 500ms
    │   ├── App.jsx                  ← Router, layout, SoundHintToast component
    │   ├── index.css                ← Tailwind directives
    │   ├── components/
    │   │   └── Sidebar.jsx          ← Nav links, version display, update alert dot
    │   └── pages/
    │       ├── Dashboard.jsx        ← 3x3 stats grid, engine status, leaderboard
    │       ├── Snippets.jsx         ← CRUD for expansions
    │       ├── Triggers.jsx         ← CRUD for triggers
    │       ├── Hotkeys.jsx          ← CRUD for hotkeys with key recorder
    │       ├── Variables.jsx        ← Custom variables + built-in popup modal
    │       ├── Settings.jsx         ← Tabbed: System, Customise, Data, Updates
    │       └── About.jsx            ← Contributors, license modal, release notes modal
    └── src-tauri/
        ├── tauri.conf.json
        ├── Cargo.toml
        └── src/
            ├── main.rs              ← calls app_lib::run()
            ├── lib.rs               ← All Tauri commands, tray setup, window management
            ├── models.rs            ← RootConfig and all data structs
            ├── engine.rs            ← Keystroke engine (rdev listener + enigo injector)
            └── helpers.rs           ← Shared persist_config() function
```

---

## 3. Cargo.toml Dependencies

```toml
[package]
name = "expandly"
version = "4.0.0"
edition = "2021"
rust-version = "1.77.2"

[lib]
name = "app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[dependencies]
serde                    = { version = "1", features = ["derive"] }
serde_json               = "1"
log                      = "0.4"
tauri                    = { version = "2.10.3", features = ["tray-icon"] }
tauri-plugin-log         = "2"
tauri-plugin-dialog      = "2"
tauri-plugin-autostart   = "2"
uuid                     = { version = "1", features = ["v4"] }
rdev                     = "0.5"
enigo                    = "0.2"
rodio                    = { version = "0.17", features = ["mp3", "wav"] }
open                     = "5"
tokio                    = { version = "1", features = ["time"] }
ureq                     = "2"
```

---

## 4. tauri.conf.json (current)

```json
{
  "identifier": "Expandly",
  "productName": "Expandly",
  "version": "4.0.0",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "Expandly 4.0.0",
        "width": 900, "height": 600,
        "minWidth": 900, "minHeight": 600,
        "resizable": true,
        "fullscreen": false,
        "visible": false,
        "label": "main"
      },
      {
        "title": "Expandly",
        "width": 400, "height": 200,
        "resizable": false,
        "decorations": false,
        "transparent": true,
        "center": true,
        "label": "splash",
        "url": "splash.html"
      }
    ],
    "security": { "csp": null }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png","icons/128x128.png","icons/128x128@2x.png","icons/icon.icns","icons/icon.ico"]
  }
}
```

**Icon usage:**
- `32x32.png` — system tray icon
- `128x128.png` — file explorer / Linux installers
- `128x128@2x.png` — high DPI / retina
- `icon.icns` — macOS app icon
- `icon.ico` — Windows taskbar, title bar, shortcut

---

## 5. models.rs — Data Structures

### RootConfig
```rust
pub struct RootConfig {
    pub version: String,
    pub enabled: bool,                           // default true
    pub sound_enabled: bool,                     // default true
    pub sound_path: Option<String>,              // supports URLs and local paths
    pub launch_at_startup: bool,
    pub launch_minimised: bool,
    pub minimise_to_tray: bool,
    pub theme: String,                           // default "starry-blue"
    pub track_stats: bool,                       // default true
    pub show_sound_hint: bool,                   // default true, dismissed after first sound play
    pub allow_prerelease: bool,                  // removed but may exist in old configs
    pub expansion_delay_ms: u64,                 // default 325
    pub buffer_size: usize,                      // default 16 (options: 16, 24, 32, 64)
    pub hotkey_delay_ms: u64,                    // default 80
    pub clear_buffer_on_switch: bool,            // default true
    pub expansions: HashMap<String, Expansion>,
    pub triggers: Vec<Trigger>,
    pub hotkeys: Vec<Hotkey>,
    pub custom_variables: Vec<CustomVariable>,
    pub stats: GlobalStats,
}
```

### Other structs
```rust
pub struct Expansion {
    pub id: String,
    pub name: String,
    pub text: String,
}

pub struct Trigger {
    pub id: String,
    pub key: String,              // e.g. "/hello" — max length = buffer_size
    pub expansion_id: String,
    pub word_boundary: bool,      // prepends " " to match key when true
}

pub struct Hotkey {
    pub id: String,
    pub keys: String,             // e.g. "Control+Shift+H"
    pub expansion_id: String,
}

pub struct CustomVariable {
    pub id: String,
    pub name: String,
    pub value: String,
}

pub struct GlobalStats {
    pub total_expansions: u64,
    pub total_chars_saved: u64,
    pub expansions_per_day: HashMap<String, u64>,  // key: "2026-03-17"
    pub expansion_counts: HashMap<String, u64>,     // key: expansion_id
}
```

---

## 6. engine.rs Architecture

### Core design
- Uses `rdev::listen` to hook all system keystrokes globally
- Runs in a background thread with a **watchdog loop** — restarts automatically on crash (1s delay)
- On each keypress, takes a quick **EngineSnapshot** (single short mutex lock)
- All heavy work (delete, inject, stats, sound) runs on **worker threads** — never blocks the hook
- Buffer tracks last N characters (configurable `buffer_size`, default 16)

### EngineSnapshot struct
```rust
struct EngineSnapshot {
    enabled, buffer_size, expansion_delay, hotkey_delay,
    clear_buffer_on_switch, sound_enabled, sound_path,
    triggers, hotkeys, expansions, custom_variables
}
```

### Trigger matching
- **Word boundary = true:** match key is ` /hello` (space prepended), space is included in delete count
- **Word boundary = false:** match key is `/hello`, matched directly
- Case insensitive via `.to_lowercase()`
- Buffer clears on Return, Escape, Tab

### Sound playback
- Supports both local file paths and HTTP/HTTPS URLs (fetched via `ureq`)
- `rodio` with mp3 + wav features
- 10 second maximum playback limit
- Fire-and-forget on worker thread

### Built-in variables (resolved at expansion time)
| Token | Output | Example |
|---|---|---|
| `{date}` | Today's date | `17/03/2026` |
| `{time}` | Current time | `14:35` |
| `{datetime}` | Date + time | `17/03/2026 14:35` |
| `{day}` | Day of week | `Friday` |
| `{month}` | Month name | `March` |
| `{year}` | Year | `2026` |
| `{hour}` | Hour only | `14` |
| `{minute}` | Minute only | `35` |
| `{yesterday}` | Yesterday's date | `16/03/2026` |
| `{tomorrow}` | Tomorrow's date | `18/03/2026` |
| `{greeting}` | Time-based greeting | `Good afternoon` |
| `{clipboard}` | Clipboard contents | (last copied text) |

Custom variables via `{name}` syntax.

---

## 7. lib.rs — Tauri Commands

Full registered command list:
- `get_config` — returns full RootConfig
- `save_config` — saves full RootConfig
- `create_expansion`, `update_expansion`, `delete_expansion`
- `create_trigger`, `update_trigger`, `delete_trigger` — enforces `buffer_size` max length
- `create_hotkey`, `update_hotkey`, `delete_hotkey`
- `create_custom_variable`, `update_custom_variable`, `delete_custom_variable`
- `update_engine_settings` — enabled, sound_enabled, sound_path
- `update_system_settings` — launch_at_startup, launch_minimised, minimise_to_tray
- `update_performance_settings` — hotkey_delay_ms, engine_restart_delay_ms, clear_buffer_on_switch
- `update_expansion_delay` — expansion_delay_ms
- `update_buffer_size` — buffer_size
- `update_track_stats` — track_stats
- `reset_stats` — clears GlobalStats
- `export_config` — opens native save dialog, writes JSON
- `reset_config` — resets to defaults
- `get_app_version` — reads from package_info (source of truth = tauri.conf.json)
- `open_url` — opens URL in system browser via `open` crate
- `save_sound_file` — saves uploaded sound bytes to app data dir
- `close_splash` — closes splash window, shows main window (respects launch_minimised)
- `dismiss_sound_hint` — sets show_sound_hint to false
- `update_prerelease_setting` — REMOVED, may still exist in some codebases

### AppState
```rust
pub struct AppState {
    pub config: Arc<Mutex<RootConfig>>,
}
```

---

## 8. Frontend Architecture

### Routing
```
/ → Dashboard
/snippets → Snippets
/triggers → Triggers
/variables → Variables
/hotkeys → Hotkeys
/settings → Settings
/about → About
```

### Navigating to a specific settings tab
```jsx
navigate('/settings', { state: { tab: 'appearance' } })
```

### Settings tabs
| ID | Label | Contents |
|---|---|---|
| `engine` | System | Enable Engine, Startup & System, Performance, Advanced |
| `appearance` | Customise | Themes (coming soon), Expansion Sound |
| `data` | Data | Statistics toggle, Backup/Import/Export, Danger Zone |
| `updates` | Updates | Update checker, release notes, changelog modal |

### Dashboard layout
- **Engine status bar** — version, active/disabled pill
- **Upper middle** — 4 cards: Snippets, Triggers, Hotkeys, Variables
- **Lower middle left** — Leaderboard (top 5 most used snippets with progress bars)
- **Lower middle right** — 3 time stats: Today, This Week, All Time

### Tauri API pattern
```jsx
// ALWAYS inside a function or useEffect — NEVER at module top level
const { invoke } = window.__TAURI_INTERNALS__
invoke('command_name', { param: value }).then(result => ...)
```

---

## 9. Key Technical Decisions & Bugs Fixed

| Issue | Fix |
|---|---|
| Top-level invoke causes white screen in production | Always call `window.__TAURI_INTERNALS__` inside functions/useEffect |
| Trailing character after expansion | Configurable expansion delay (default 325ms) |
| Word boundary adds trailing space | Space is prepended to match key and included in delete count |
| Case-sensitive triggers | `.to_lowercase()` comparison |
| Double tray icon | Remove `trayIcon` from tauri.conf.json — tray created in code only |
| Splash screen shows main window briefly | Main window has `"visible": false`, shown only after `close_splash` |
| Theme not persisting | Loaded in main.jsx before React renders |
| Full Tailwind theme system | Deferred to 4.1 — themes currently use CSS variables but Tailwind classes are hardcoded |
| External links in WebView | Use `invoke('open_url', { url })` not `<a>` tags |
| Sound URL support | `play_sound()` detects `http://`/`https://` and fetches via `ureq` |
| `window.confirm` not working in Tauri | Use custom `ConfirmModal` React component instead |
| `update_prerelease_setting` removed | Remove from invoke handler if it throws compile errors |

---

## 10. App Defaults (new installs)

```json
{
  "version": "4.0.0",
  "enabled": true,
  "sound_enabled": true,
  "sound_path": "https://cdn.klazorix.com/expandly/default_sound.mp3",
  "show_sound_hint": true,
  "track_stats": true,
  "expansion_delay_ms": 325,
  "buffer_size": 16,
  "hotkey_delay_ms": 80,
  "clear_buffer_on_switch": true,
  "theme": "starry-blue"
}
```

Default snippets: "Welcome to Expandly" (`/hello`), "Current Date & Time" (`/time`)
Default custom variable: `{version}` = `4.0.0`

---

## 11. Update Checker System

- Fetches `https://api.github.com/repos/klazorix/Expandly/releases/latest` for stable
- Compares `tag_name` against installed version using `newerVersion()` function
- `newerVersion` strips leading `v`, splits by `.`, compares position by position left-to-right
- Handles pre-release format `4.0.0b1` — base version compared first, then pre suffix
- Sidebar shows orange `AlertCircle` dot if update available
- Release notes rendered with `react-markdown` + `remark-gfm` in `ChangelogModal`
- `formatDate`, `formatBytes` are helper functions in Settings.jsx

---

## 12. Statistics System

- `track_stats` toggle in Settings → Data
- Disabling prompts custom `ConfirmModal` (not `window.confirm`) and clears all stats via `reset_stats`
- Stats recorded fire-and-forget on worker thread in `engine.rs`
- `expansion_counts` per expansion ID enables leaderboard
- Dashboard polls `get_config` every 5 seconds for live stats
- Warning shown on Dashboard when `track_stats` is false

---

## 13. Performance Settings

Located in Settings → System → Performance:
- **Expansion Delay** — dropdown, 250-750ms in 50ms steps (moved to Advanced modal)
- **Buffer Size** — dropdown: 16, 24, 32, 64 chars
- **Clear Buffer on Window Switch** — toggle

Located in Advanced modal (Settings → System → Advanced):
- **Expansion Delay** — number input, ms
- **Hotkey Inject Delay** — number input, ms
- Warning: "Only users who fully understand the potential impact should modify these settings."

---

## 14. Splash Screen

- `public/splash.html` — standalone HTML with inline CSS, no Vite/React
- Shows logo (`icon.png` from `public/`), "Expandly 4" text, animated dots
- Transparent, borderless, centered, 400x200
- Main window has `"visible": false` in tauri.conf.json
- `main.jsx` calls `invoke('close_splash')` after 500ms delay once React renders
- `close_splash` in lib.rs closes splash, shows main (respects `launch_minimised`)

---

## 15. GitHub Actions Build

Workflow file at `.github/workflows/build.yml`:
- Triggers on version tags (`v*`) or manual dispatch
- Builds on `windows-latest`, `macos-latest`, `ubuntu-22.04`
- macOS targets `aarch64-apple-darwin`
- Linux requires: `libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev libxtst-dev libx11-dev`
- Uses `tauri-apps/tauri-action@v0`
- `projectPath: expandly-4.0.0`
- Creates draft GitHub Release with installers attached
- Trigger: `git tag v4.0.0 && git push origin v4.0.0`

---

## 16. Contributors

- **klazorix** — Developer
- **encryptednoobi** — Logo Designer

---

## 17. npm Packages

```json
{
  "dependencies": {
    "react": "^19",
    "react-dom": "^19",
    "react-router-dom": "^7",
    "lucide-react": "latest",
    "react-markdown": "latest",
    "remark-gfm": "latest"
  },
  "devDependencies": {
    "vite": "^8",
    "tailwindcss": "^4",
    "@tailwindcss/postcss": "latest",
    "@tailwindcss/typography": "latest"
  }
}
```

---

## 18. Conventions & Rules

- **No top-level invoke** — always inside component functions or hooks
- **No `<a>` tags for external links** — use `invoke('open_url', { url })` instead
- **No hardcoded version strings** — use `invoke('get_app_version')`
- **Themes deferred** — CSS variables exist but Tailwind classes are hardcoded; full theme system in 4.1
- **JSX not TSX** — project is JavaScript, not TypeScript
- **Rust naming** — snake_case for fields; camelCase for JS invoke parameters
- **File outputs** — downloadable files go to `/mnt/user-data/outputs/` when Claude generates them
- **Modals instead of window.confirm** — `window.confirm` doesn't work in Tauri WebView
- **`window.__TAURI_INTERNALS__`** — always destructure inside functions, never at module level

---

## 19. Planned Future: Expandly 4.0.0 "Launch"

This is the next release. **Backend and performance changes only — no new UI features.**

### Changes
1. **Migrate to SQLite** — replace `config.json` with a SQLite database; each domain (snippets, triggers, hotkeys, variables, settings, stats) gets its own table; `Arc<Mutex<Connection>>` replaces `Arc<Mutex<RootConfig>>`
2. **Import/Export** — export queries all tables → single JSON file; import reads JSON → populates tables
3. **Codebase restructure** — shared types defined once, split lib.rs into domain modules, remove dead code
4. **Frontend optimisation** — consolidate shared components, extract reusable hooks
5. **Final optimisation pass** — performance profiling, clean comments

### Note on architecture
When implementing SQL: UI commands write to the relevant Arc AND file without touching others. Engine reads only what it needs via targeted queries. No polling — changes are live immediately via shared Arc.

### Reminder for this work
Restructure how aspects of the codebase are made so things are only defined once (especially RootConfig and shared structs) and can be accessed across multiple files via imports.

---

## 20. Full Release Roadmap

### Expandly 4.x

| Version | Theme | Features |
|---|---|---|
| **4.0.0** | **Launch** | SQL migration, import/export merged JSON, codebase restructure |
| 4.1 | Personalise | Themes, onboarding wizard |
| 4.2 | Organise | Snippet folders/categories, snippet tagging and filtering |
| 4.3 | Manage | Bulk actions, snippet pinning and duplicating, snippet enabled/disabled toggle |
| 4.4 | Archive | Snippet versioning (SQL toggle), snippet templates, snippet sharing/export |
| 4.5 | Discover | Snippet search bar, command palette, live preview with variables resolved |
| 4.6 | Detect | Trigger and hotkey conflict detection, multi-line trigger support |
| 4.7 | Activate | Choice menu popup on trigger, undo last expansion |
| 4.8 | Control | Custom tray icons + pause/resume from tray, keyboard shortcut to open window |
| 4.9 | Connect | Import from other text expanders, per-app override configs foundation |
| 4.10 | Enrich | Rich text support, emoji system |
| 4.11 | Trace | Expansion history log, per-snippet analytics |
| 4.12 | Polish | Debounce for mechanical keyboards |

### Expandly 5.x

| Version | Theme | Features |
|---|---|---|
| **5.0** | **Evolve** | Snippet types (text, shell, script), `{{name}}` variable syntax, app-specific blacklists/whitelists, per-app override configs, multi-platform (macOS, Linux), possible UI rework |
| 5.1 | Logic | Random selection variable, logic/conditionals variable |
| 5.2 | Forms | Forms, automatic cursor placement |
| 5.3 | Automate | Scheduled expansions, clipboard history integration |
| 5.4 | Pattern | Regex triggers, voice triggers |
| 5.5 | Insight | Expansion history revisited (with app context), per-snippet analytics revisited |
| 5.6 | Refine | Snippet templates revisited, further engine optimisations for new snippet types |
| 5.7 | Extend | Additional snippet types (API calls, dynamic content), shell and scripts integration |

---

## 21. Development History Summary

The project started as "Text Expander v4.0.0" in March 2026, built from scratch. Here is a condensed timeline of what was built:

1. **Data architecture** — models.rs with RootConfig, Expansion, Trigger, Hotkey, CustomVariable, GlobalStats
2. **Project scaffold** — Tauri v2 + Vite React + Tailwind CSS v4 setup on Windows
3. **Rust backend** — lib.rs with persist_config, load_or_create_config, all CRUD commands
4. **React UI shell** — React Router, Sidebar with navigation, all page scaffolds
5. **Snippets page** — full CRUD with modal editor
6. **Triggers page** — CRUD with word boundary toggle, max length enforcement
7. **Hotkeys page** — CRUD with key combination recorder
8. **Variables page** — custom variables CRUD + built-in variables reference popup
9. **Dashboard** — stats grid, engine status, leaderboard, time-based stats
10. **Settings page** — tabbed: System (engine, performance, advanced), Customise (themes, sound), Data (stats, backup, danger), Updates
11. **Engine** — rdev global keyboard hook, enigo text injection, trigger matching, hotkey matching, variable resolution, sound playback, stats recording, watchdog restart loop, EngineSnapshot optimisation
12. **System tray** — tray icon, left-click toggle, right-click menu (Show Window, Quit)
13. **Autostart** — tauri-plugin-autostart integration
14. **About page** — contributors with GitHub avatars, license modal, release notes modal
15. **Splash screen** — loading screen before main window appears
16. **Statistics** — total expansions, chars saved, per-day tracking, per-expansion counts, leaderboard
17. **Update checker** — GitHub API, version comparison, changelog modal with react-markdown
18. **Sound system** — upload sound, URL support, play on expansion, hint toast on first use
19. **Performance settings** — expansion delay, buffer size, clear buffer on switch, hotkey delay, advanced modal
20. **Word boundary fix** — space prepended to match key instead of checking previous character
21. **Renamed** from TextExpander to Expandly, repo at klazorix/Expandly
