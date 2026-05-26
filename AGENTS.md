# Clipboard App

> macOS clipboard history manager — Tauri v2 + React 18. Fills the Win+V gap.

---

## Project Tree

```
clipboard-app/
├── index.html
├── package.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
├── tsconfig.json
├── src/
│   ├── main.tsx                          # React entry
│   ├── App.tsx                           # Shell: layout, keyboard, events, settings toggle
│   ├── App.css                           # Root styles, border-radius, dark color-scheme
│   ├── vite-env.d.ts
│   ├── types/
│   │   └── index.ts                      # ClipboardItem, ContentType, ClipboardStore
│   ├── stores/
│   │   └── clipboardStore.ts             # Zustand store + IPC + event listener
│   ├── hooks/
│   │   └── useDebounce.ts                # Generic debounce hook
│   ├── components/
│   │   ├── SearchBar.tsx                  # Auto-focus, 200ms debounced search, clear btn
│   │   ├── ClipboardList.tsx             # @tanstack/react-virtual, loading/empty/item states
│   │   ├── ClipboardItem.tsx             # Row: preview, time-ago, pin/delete actions
│   │   ├── Footer.tsx                    # Item count, clear-all, settings gear
│   │   └── SettingsPanel.tsx             # Overlay: config, clear, about
│   └── styles/
│       └── globals.css                   # Tailwind base, custom scrollbar, system font
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── capabilities/default.json
    └── src/
        ├── main.rs                       # Entry: calls lib::run()
        ├── lib.rs                        # Builder: plugins, setup, shortcut, commands
        ├── db/
        │   ├── mod.rs
        │   └── models.rs                 # SQLite schema, CRUD, ClipboardItem struct
        ├── clipboard/
        │   ├── mod.rs
        │   └── monitor.rs                # 500ms polling, dedup, DB insert, event emit
        ├── commands/
        │   └── mod.rs                    # 5 Tauri commands (IPC)
        └── paste/
            └── mod.rs                    # arboard set + osascript Cmd+V
```

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                       Tauri App                              │
│                                                              │
│  ┌──────────────────────┐    ┌────────────────────────────┐  │
│  │   React Frontend     │    │       Rust Backend          │  │
│  │   (WebView)          │    │                             │  │
│  │                      │    │  clipboard/monitor.rs       │  │
│  │  App.tsx             │    │    poll arboard q500ms      │  │
│  │   ├─ SearchBar       │◄───│    dedup → db.insert()      │  │
│  │   ├─ ClipboardList   │    │    emit("clipboard-new-item")│  │
│  │   │   └─ Virtualizer │    │                             │  │
│  │   ├─ ClipboardItem   │    │  db/models.rs               │  │
│  │   ├─ Footer          │    │    SQLite (clipboard.db)    │  │
│  │   └─ SettingsPanel   │    │    insert/get_all/delete/   │  │
│  │                      │    │    toggle_pin/clear/purge   │  │
│  │  stores/             │    │                             │  │
│  │   clipboardStore.ts──┼───►│  commands/mod.rs            │  │
│  │     invoke("get_..") │    │    get_history               │  │
│  │     invoke("delete") │    │    delete_item               │  │
│  │     invoke("toggle") │    │    toggle_pin               │  │
│  │     invoke("clear")  │    │    clear_history             │  │
│  │     invoke("copy_..")│    │    copy_and_paste            │  │
│  │                      │    │                             │  │
│  │                      │    │  paste/mod.rs               │  │
│  │                      │    │    arboard.set_text()        │  │
│  │                      │    │    osascript Cmd+V          │  │
│  │                      │    │                             │  │
│  │                      │    │  lib.rs                     │  │
│  │                      │    │    global shortcut handler   │  │
│  │                      │    │    window position (top-r)  │  │
│  └──────────────────────┘    └────────────────────────────┘  │
│                                                              │
│              ↕  IPC: invoke() + listen() events               │
└──────────────────────────────────────────────────────────────┘
```

---

## Data Flow

```
COPY (Cmd+C in any app)
  │
  ▼
monitor.rs poll loop (q500ms)
  │
  ├─ arboard.get_text()
  ├─ dedup: text != last_content ?
  ├─ create_clipboard_item() → UUID v4 + chrono::Utc timestamp
  ├─ db.insert(item)
  ├─ db.purge_oldest(500)
  └─ app.emit("clipboard-new-item", item)
       │
       ▼
  clipboardStore.ts listener
       │
       └─ setState: items = [newItem, ...items]

PRESS (Cmd+Shift+V anywhere)
  │
  ▼
lib.rs global shortcut handler → ShortcutState::Pressed
  │
  ├─ position_window_top_right()  (primary monitor, -16px right, +32px top)
  ├─ window.show()
  ├─ window.set_focus()
  └─ App.tsx on mount → fetchHistory()

NAVIGATE (↑↓ Enter)
  │
  ▼
App.tsx handleKeyDown()
  │
  ├─ Escape    → window.hide()
  ├─ ArrowDown → selectedIndex + 1
  ├─ ArrowUp   → selectedIndex - 1
  └─ Enter     → copyAndPaste(items[selectedIndex].id)

PASTE (click or Enter on item)
  │
  ▼
commands::copy_and_paste(id)
  │
  ├─ db.get_all() → find item by id
  ├─ paste::copy_and_paste(item.content)
  │     ├─ arboard.set_text(content)
  │     ├─ sleep 100ms
  │     └─ osascript -e 'tell app "System Events" to keystroke "v" using command down'
  └─ window.hide()
```

---

## Window Config

| Property | Value | File |
|----------|-------|------|
| Size | 360 × 480 | `tauri.conf.json` |
| Decorations | `false` (frameless) | `tauri.conf.json` |
| Always on top | `true` | `tauri.conf.json` |
| Background | Transparent + sidebar vibrancy | `tauri.conf.json` |
| Border radius | `12px` | `App.css` |
| Position | Top-right (computed at show) | `src-tauri/src/lib.rs:23-32` |
| Visibility | Starts hidden, toggled by shortcut | `tauri.conf.json` + `lib.rs` |
| Hide triggers | Esc key, blur, close request, paste | `App.tsx:28`, `App.tsx:44-49`, `commands/mod.rs:41` |

---

## Rust Backend Reference

### `lib.rs` — Entry Point
```
Path: src-tauri/src/lib.rs
```
- `OnceLock<AppHandle>` static for access from shortcut handler closure
- `toggle_window()` — shows (with positioning) or hides the main window
- `position_window_top_right()` — 16px from right edge, 32px from top (menu bar)
- Builder: `tauri_plugin_opener`, `tauri_plugin_global_shortcut` with `with_handler`
- Shortcut: `Modifiers::SUPER | Modifiers::SHIFT` + `Code::KeyV`
- Setup: inits DB dir, creates `Database`, manages state, starts clipboard monitor
- Commands registered: `get_history`, `delete_item`, `toggle_pin`, `clear_history`, `copy_and_paste`

### `db/models.rs` — SQLite Layer
```
Path: src-tauri/src/db/models.rs
```
- `ClipboardItem` struct — derives `Serialize`, `Deserialize`, `Clone`, `Debug`
- `Database` struct — wraps `Mutex<Connection>`
- `Database::new(path)` — opens connection, runs `CREATE TABLE IF NOT EXISTS` + indexes
- `insert(item)` — INSERT INTO clipboard_history
- `get_all(search?)` — SELECT with optional `WHERE content LIKE ?1 OR preview LIKE ?1`, orders by `is_pinned DESC, created_at DESC`
- `delete(id)` — DELETE by id
- `toggle_pin(id)` — UPDATE is_pinned = NOT is_pinned, returns new bool
- `clear_unpinned()` — DELETE WHERE is_pinned = 0
- `purge_oldest(keep)` — DELETE oldest unpinned beyond limit
- `create_clipboard_item()` — standalone constructor: UUID + RFC3339 timestamp

**Schema:**
```sql
CREATE TABLE clipboard_history (
  id          TEXT PRIMARY KEY,
  content     TEXT NOT NULL,
  content_type TEXT NOT NULL DEFAULT 'text',
  preview     TEXT NOT NULL DEFAULT '',
  app_name    TEXT,
  is_pinned   INTEGER NOT NULL DEFAULT 0,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
);
CREATE INDEX idx_created_at ON clipboard_history(created_at DESC);
CREATE INDEX idx_pinned     ON clipboard_history(is_pinned DESC);
```

### `clipboard/monitor.rs` — Polling Watcher
```
Path: src-tauri/src/clipboard/monitor.rs
```
- Spawned as `tauri::async_runtime::spawn` in setup
- Gets `Database` state via `app.state::<Database>()`
- Loop: `arboard::Clipboard::get_text()` → dedup → db.insert() → db.purge_oldest(500) → emit event
- Preview: first 120 chars
- Constant: `MAX_HISTORY = 500`

### `commands/mod.rs` — Tauri IPC Commands
```
Path: src-tauri/src/commands/mod.rs
```

| Command | Args | Returns | Side Effects |
|---------|------|---------|-------------|
| `get_history` | `search?: String` | `Vec<ClipboardItem>` | Reads DB |
| `delete_item` | `id: String` | `()` | Deletes from DB |
| `toggle_pin` | `id: String` | `bool` | Updates DB, returns new pin state |
| `clear_history` | — | `()` | Deletes all unpinned |
| `copy_and_paste` | `id: String` | `()` | DB lookup → set clipboard → osascript paste → hide window |

### `paste/mod.rs` — Paste Agent
```
Path: src-tauri/src/paste/mod.rs
```
1. `arboard::Clipboard::set_text(content)`
2. `thread::sleep(100ms)` — ensure clipboard is ready
3. `Command::new("osascript").args(["-e", "tell application \"System Events\" to keystroke \"v\" using command down"])`

---

## Frontend Reference

### `src/stores/clipboardStore.ts` — Zustand Store
```
Path: src/stores/clipboardStore.ts
```
**State:** `items`, `searchQuery`, `selectedIndex`, `isLoading`

**Actions (all via `invoke` from `@tauri-apps/api/core`):**
| Action | IPC Call | Notes |
|--------|----------|-------|
| `fetchHistory(search?)` | `get_history` | Sets items state |
| `copyAndPaste(id)` | `copy_and_paste` | Window hides server-side |
| `deleteItem(id)` | `delete_item` | Optimistic removal from state |
| `togglePin(id)` | `toggle_pin` | Optimistic flip in state |
| `clearHistory()` | `clear_history` | Clears state |
| `setSearch(q)` | — | Local, resets selectedIndex |
| `setSelectedIndex(i)` | — | Local |

**`initClipboardListener()`** — calls `listen("clipboard-new-item")` from `@tauri-apps/api/event`, prepends new items to state.

### `src/App.tsx` — App Shell
```
Path: src/App.tsx
```
- Mount: init clipboard listener, fetch history, set up blur/close handlers
- Window blur → `appWindow.hide()`
- Window close-requested → `appWindow.hide()` (never destroy window)
- Keyboard handler: `Escape` (hide), `ArrowDown/Up` (navigate), `Enter` (paste)
- Search query change → `fetchHistory(searchQuery)` with debounce applied by SearchBar
- Client-side filter displayed in list
- Settings panel toggle via state

### `src/components/SearchBar.tsx`
```
Path: src/components/SearchBar.tsx
```
- Auto-focus via `useRef` + `useEffect`
- Local `input` value for instant keystroke feedback
- 200ms debounce on `setSearch()` via `setTimeout`/`clearTimeout`
- Clear button (× icon) when query is non-empty
- SVG search icon (magnifying glass) inline
- Styling: `bg-white/10` → `hover:bg-white/15` → `focus:bg-white/20`

### `src/components/ClipboardList.tsx`
```
Path: src/components/ClipboardList.tsx
```
- `useVirtualizer` from `@tanstack/react-virtual`
- `estimateSize: 44px` per item
- Absolute positioning in scroll container for virtual rows
- `scrollToIndex(selectedIndex)` on keyboard nav changes
- States: loading (5 skeleton rows with `animate-pulse`), empty (clipboard icon + "No clipboard history yet"), items (virtualized list)

### `src/components/ClipboardItem.tsx`
```
Path: src/components/ClipboardItem.tsx
```
- Displays: `preview` (truncated), time-ago label, pin button, delete button
- `timeAgo()` helper: `just now` / `Xm ago` / `Xh ago` / `Xd ago`
- Selected highlight: `bg-white/20`
- Hover: `bg-white/10`, action buttons appear via `opacity` transition
- Pin icon: yellow fill when pinned, dim when not
- Delete icon: turns red on hover
- Click row → `copyAndPaste(item.id)`
- Stop propagation on pin/delete button clicks

### `src/components/Footer.tsx`
```
Path: src/components/Footer.tsx
```
- Item count: "N items" / "N item"
- "Clear all" button with `confirm()` dialog
- Settings gear icon → calls `onOpenSettings` prop

### `src/components/SettingsPanel.tsx`
```
Path: src/components/SettingsPanel.tsx
```
- Full-screen overlay (`absolute inset-0 z-50`)
- Sections: Max items (500), stored count, Clear all (two-step confirm), Shortcut (Cmd+Shift+V), About (v0.1.0)
- Confirm-clear: first click shows "Confirm clear all" in red, second click executes

---

## User Interactions

| Action | Trigger | Behavior |
|--------|---------|----------|
| Open window | `Cmd+Shift+V` (global) | Window appears top-right, search focused |
| Close window | `Esc`, blur, or `Cmd+Shift+V` again | Window hides (not destroyed) |
| Search | Type in search bar | 200ms debounce → `get_history(search)` → filtered list |
| Navigate | `↑` `↓` arrows | Moves `selectedIndex`, highlights row, scrolls into view |
| Paste item | `Enter` or click row | Copies to clipboard → auto-pastes to active app → hides window |
| Pin item | Click pin icon | Toggles `is_pinned`, pinned items sort first |
| Delete item | Click × icon | Removes from DB and list |
| Clear all | Footer "Clear all" or Settings | `confirm()` dialog → deletes all unpinned |
| Settings | Footer gear icon | Overlay with config/about |

---

## Cargo Dependencies (src-tauri/Cargo.toml)
```toml
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
tauri-plugin-global-shortcut = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
arboard = "3"
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["time"] }
```

## NPM Dependencies (package.json)
```json
{
  "dependencies": {
    "react": "^18.3.1",
    "react-dom": "^18.3.1",
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-opener": "^2",
    "zustand": "^5.0.13",
    "@tanstack/react-virtual": "^3.13.26"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@types/react": "^18.3.1",
    "@types/react-dom": "^18.3.1",
    "@vitejs/plugin-react": "^4.3.4",
    "typescript": "~5.6.2",
    "vite": "^6.0.3",
    "tailwindcss": "^3.4.19",
    "postcss": "^8.5.15",
    "autoprefixer": "^10.5.0"
  }
}
```

---

## Build & Run

```bash
# Install frontend deps
pnpm install

# Run in dev mode (hot-reload)
pnpm tauri dev

# Build for production
pnpm tauri build

# TypeScript check only
npx tsc --noEmit

# Rust check only
cargo build --manifest-path src-tauri/Cargo.toml
```

---

## Constraints & Decisions

1. **Polling not push**: macOS has no native clipboard-change notification. 500ms polling is the standard approach (Alfred, Paste, etc.).
2. **SQLite over flat files**: Indexed search, transaction safety, future-proof for images/metadata.
3. **osascript for paste**: Simulating Cmd+V via AppleScript is the only reliable cross-app paste method on macOS. 100ms delay ensures clipboard is set before keystroke.
4. **Frameless + vibrancy**: `decorations: false`, `transparent: true`, `windowEffects: ["sidebar"]` in `tauri.conf.json`. CSS `border-radius: 12px` on `#root`.
5. **Max 500 items**: Auto-purges oldest unpinned entries on insert. Hardcoded in `monitor.rs:8`.
6. **Window lifecycle**: Never destroyed — only shown/hidden. Close-requested → hide. Blur → hide. Paste → hide. Persists between toggles.
7. **Image support**: Not yet implemented. Schema supports `content_type` field; would store base64 or file path.
8. **Security**: All data stored locally in `~appData/clipboard.db`. No network calls. Clipboard content may contain sensitive data (passwords, tokens).
