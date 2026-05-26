# Clipboard App — macOS Clipboard History

> A minimalist clipboard history manager for macOS built with Tauri v2 + React 18.
> macOS lacks a built-in clipboard history (unlike Windows Win+V). This app fills that gap.

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Framework | Tauri v2 (Rust backend) |
| Frontend | React 18 + TypeScript |
| Bundler | Vite 6 |
| Styling | Tailwind CSS 3 |
| State | Zustand |
| Virtual List | @tanstack/react-virtual |
| DB | SQLite via rusqlite (bundled) |
| Clipboard | arboard crate |
| Global Shortcut | tauri-plugin-global-shortcut |

---

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  Tauri App                       │
│  ┌──────────────────┐  ┌─────────────────────┐  │
│  │  React Frontend   │  │   Rust Backend      │  │
│  │  (WebView)        │  │                     │  │
│  │  ┌────────────┐   │  │  ┌──────────────┐  │  │
│  │  │ SearchBar   │   │  │  │ Clipboard    │  │  │
│  │  │ (auto-focus)│   │  │  │ Watcher      │  │  │
│  │  └────────────┘   │  │  │ (500ms poll) │  │  │
│  │  ┌────────────┐   │  │  └──────────────┘  │  │
│  │  │ VirtualList │   │  │  ┌──────────────┐  │  │
│  │  │ (infinite)  │   │  │  │ SQLite Store │  │  │
│  │  └────────────┘   │  │  └──────────────┘  │  │
│  │  ┌────────────┐   │  │  ┌──────────────┐  │  │
│  │  │ Preview    │   │  │  │ Paste Agent  │  │  │
│  │  │ (inline)   │   │  │  │ (osascript)  │  │  │
│  │  └────────────┘   │  │  └──────────────┘  │  │
│  └──────────────────┘  └─────────────────────┘  │
│           ↕ Tauri IPC (invoke + events)          │
└─────────────────────────────────────────────────┘
```

---

## Data Flow

```
1. User copies something (Cmd+C / any app)
2. Rust watcher detects new text every 500ms
3. Deduplicate: skip if same as last entry
4. Insert into SQLite with UUID, timestamp, content_type
5. Emit event to React UI → list updates in real-time
6. User presses Cmd+Shift+V → window opens, search focused
7. User types to filter, uses ↑↓ + Enter to navigate
8. Click / Enter on item:
   a. Rust: write content back to system clipboard
   b. Rust: run osascript Cmd+V (auto-paste to active app)
   c. Rust: close window
```

---

## Window Spec

| Property | Value |
|----------|-------|
| Width | 360px |
| Height | 480px |
| Decorations | `false` (frameless) |
| Always-on-top | `true` |
| Background | macOS vibrancy blur effect |
| Position | Top-right corner (Spotlight-like) |
| Hotkey | `Cmd+Shift+V` |
| Behavior | Toggle open/close (hide on blur or after paste) |

---

## Data Models

### SQLite Schema
```sql
CREATE TABLE clipboard_history (
  id TEXT PRIMARY KEY,
  content TEXT NOT NULL,
  content_type TEXT NOT NULL DEFAULT 'text',
  preview TEXT NOT NULL DEFAULT '',
  app_name TEXT,
  is_pinned INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX idx_created_at ON clipboard_history(created_at DESC);
CREATE INDEX idx_pinned ON clipboard_history(is_pinned DESC);
```

### TypeScript Types
```typescript
type ContentType = 'text' | 'image' | 'file' | 'rich_text';

interface ClipboardItem {
  id: string;
  content: string;        // text content / base64 for images
  content_type: ContentType;
  preview: string;         // first 120 chars / filename / "[Image]"
  app_name?: string;
  is_pinned: boolean;
  created_at: string;      // ISO 8601
  updated_at: string;
}

interface ClipboardStore {
  items: ClipboardItem[];
  searchQuery: string;
  selectedIndex: number;
  isLoading: boolean;
  fetchHistory: () => Promise<void>;
  copyAndPaste: (id: string) => Promise<void>;
  deleteItem: (id: string) => Promise<void>;
  togglePin: (id: string) => Promise<void>;
  clearHistory: () => Promise<void>;
  setSearch: (q: string) => void;
  setSelectedIndex: (i: number) => void;
}
```

### Rust Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub content: String,
    pub content_type: ContentType,
    pub preview: String,
    pub app_name: Option<String>,
    pub is_pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType { Text, Image, File, RichText }
```

---

## Rust Backend — Modules

### `clipboard/monitor.rs`
- Polls system clipboard every 500ms via `arboard`
- Compares with last known content (dedup)
- On new content: determines type (text/image/file), generates preview, saves to DB
- Emits `clipboard-new-item` event to frontend

### `db/models.rs`
- SQLite init (`CREATE TABLE IF NOT EXISTS`)
- CRUD operations: `insert`, `get_all`, `search`, `delete`, `toggle_pin`, `clear`
- Config: store last N items (default 500), auto-purge oldest unpinned

### `commands/mod.rs`
| Command | Args | Returns | Description |
|---------|------|---------|-------------|
| `get_history` | `search?: String` | `Vec<ClipboardItem>` | Fetch history, pinned first, newest first |
| `copy_and_paste` | `id: String` | `()` | Set clipboard + simulate Cmd+V |
| `delete_item` | `id: String` | `()` | Delete single item |
| `toggle_pin` | `id: String` | `()` | Toggle `is_pinned` |
| `clear_history` | — | `()` | Delete all unpinned items |

### `paste/mod.rs`
```rust
// Auto-paste via AppleScript
Command::new("osascript")
    .args(["-e", "tell app \"System Events\" to keystroke \"v\" using command down"])
    .output()
```
- Small delay before paste to ensure clipboard is set
- Fallback: just copy to clipboard (user pastes manually)

---

## Frontend — Components

| Component | File | Responsiblity |
|-----------|------|--------------|
| `App` | `App.tsx` | Layout shell, event listeners, global keyboard |
| `SearchBar` | `components/SearchBar.tsx` | Auto-focused input, debounced search |
| `ClipboardList` | `components/ClipboardList.tsx` | Virtualized list, keyboard nav container |
| `ClipboardItem` | `components/ClipboardItem.tsx` | Single row: icon, preview, timestamp, actions |
| `EmptyState` | `components/EmptyState.tsx` | "No history yet" + animation |
| `SettingsPanel` | `components/SettingsPanel.tsx` | Max items, clear all, theme toggle |

### UI States per Component
| Component | States |
|-----------|--------|
| `SearchBar` | Default (placeholder), Focused (glow), HasQuery (clear btn) |
| `ClipboardList` | Loading (skeleton), Empty, HasItems, Filtered (search active), NoResults |
| `ClipboardItem` | Default, Hovered (bg change), Selected (keyboard nav), Pinned (icon fill) |
| `EmptyState` | Absolute first launch vs all cleared |

---

## UI/UX Design

### Layout
```
┌──────────────────────┐
│ 🔍 Search history... │  ← rounded search bar
├──────────────────────┤
│ 📄 "Hello world..."  │  ← item card
│   just now           │  [📌] [✕]
├──────────────────────┤
│ 📄 "Some other..."   │
│   5 min ago          │  [📌] [✕]
├──────────────────────┤
│                      │
│  3 items · Clear all  │  ← footer
└──────────────────────┘
```

### Visual Style
- Frameless window with `border-radius: 12px`
- macOS vibrancy (`NSVisualEffectView` behind webview)
- Dark/light mode following system
- Subtle shadows, minimal borders
- Monochrome icons, clean typography (system font - SF Pro)
- Item cards: subtle hover background, active press state
- Smooth transitions (opacity, translate)

### Interactions
- `Click` → copy + paste + close
- `Cmd+Click` → copy only, stay open
- `↑↓` arrows → navigate list
- `Enter` → confirm selection
- `Esc` → close window
- `⌘+Shift+V` (global) → toggle open/close
- Pin icon → toggle pinned state
- Delete icon → remove item
- Click "Clear all" → confirmation → delete all

---

## Dependencies

### Rust (`Cargo.toml`)
```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
tauri-plugin-global-shortcut = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
arboard = "3"
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Frontend (`package.json`)
```json
{
  "dependencies": {
    "react": "^18.3.1",
    "react-dom": "^18.3.1",
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-opener": "^2",
    "zustand": "^4.5",
    "@tanstack/react-virtual": "^3.5"
  },
  "devDependencies": {
    "tailwindcss": "^3.4",
    "postcss": "^8",
    "autoprefixer": "^10",
    "@tauri-apps/cli": "^2"
  }
}
```

---

## Implementation Order

| # | Step | Description | Est. |
|---|------|-------------|------|
| 1 | Init scaffolding | Install Tailwind, Zustand, React Virtual; configure CSS | 15m |
| 2 | Rust: DB layer | SQLite schema, CRUD functions | 30m |
| 3 | Rust: clipboard watcher | arboard polling loop, dedup, insert | 45m |
| 4 | Rust: Tauri commands | Wire get_history, delete, toggle_pin, clear, copy_and_paste | 30m |
| 5 | React: Zustand store | Types, store, IPC invocation hooks | 30m |
| 6 | React: UI shell | Frameless window, dark mode, layout | 45m |
| 7 | React: SearchBar | Auto-focus, debounced search, escape handling | 20m |
| 8 | React: VirtualList + Items | Virtualized list with keyboard nav | 45m |
| 9 | Rust: paste agent | osascript Cmd+V simulation | 15m |
| 10 | Rust: global shortcut | Cmd+Shift+V toggle via plugin | 20m |
| 11 | React: polish | Empty state, skeletons, transitions, edge cases | 45m |
| 12 | Settings | Max items config, clear history, about | 30m |

---

## Known Decisions & Constraints

1. **Polling not event-driven**: macOS has no native clipboard change notification. 500ms polling is the standard approach (used by Alfred, Paste, etc.).
2. **SQLite over JSON**: Better perf for search, indexing, and future-proofing.
3. **osascript for paste**: Simulating Cmd+V via AppleScript is the only reliable cross-app paste method on macOS.
4. **Frameless + transparent**: Requires macOS `fullScreenEnabled` and vibrancy config in Tauri.
5. **Max history**: Default 500 items. Configurable. Auto-purges oldest unpinned on insert when full.
6. **Image support**: Stretch goal. Store as base64 in DB (or file path). Needs thumbnail generation.
7. **Security**: Clipboard content may contain sensitive data (passwords). All storage is local-only, no network.
