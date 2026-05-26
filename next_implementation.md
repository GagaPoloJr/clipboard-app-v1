# Next Improvements & Features

## Just Implemented

- **[x] Cursor-aware window positioning** — window now appears on whichever monitor the cursor is on, not always the primary monitor. Falls back to primary if cursor position detection fails.

---

## High Priority

### 1. Image Clipboard Support

Currently only text clipboard works. Extend to support images:

- **Rust**: Detect image content type in `monitor.rs` via `arboard.get_image()`, store as PNG bytes or file path in DB
- **Rust**: Extend `paste/mod.rs` to handle image paste (osascript for images is trickier)
- **Frontend**: Show image thumbnail preview in `ClipboardItem`
- **DB**: Already has `content_type` column — just need to populate it

### 2. Rich Text / HTML Support

Detect and preserve formatted content (RTF, HTML from browsers/pages):

- Store `text/html` alongside plain text
- Render rich preview in the item row (strip tags for preview)

### 3. Copy Feedback Toast (done)

Show a brief "Copied!" notification when clicking an item:

- Toast overlay in bottom of window, auto-dismiss after 1.5s
- Tiny animation (slide up, fade out)
- Component: `src/components/Toast.tsx`

### 4. Search Highlight

Highlight matching text in item previews when a search query is active:

- Parse search query, wrap matches in `<mark>` or `<span>` with highlight class
- Use `text-yellow-200` or similar for highlights

### 5. Window Show Animation

Smooth entry animation when the window appears (currently instant):

- Add a subtle scale + opacity transition on show
- CSS: `animate-in` with `@keyframes`
- Consider `requestAnimationFrame` approach

---

## Medium Priority

### 6. Paste Without Auto-Close

Currently `copy_and_paste` hides the window. Add:

- **Cmd+Click** or **Shift+Click** on item: copy + paste + keep window open
- Right-click context menu: "Copy", "Copy & Paste", "Pin", "Delete"

### 7. Keyboard Shortcut Customization

Let users change the global shortcut:

- Settings UI: input field that captures key combination
- Rust: unregister current shortcut, register new one
- Store preference in DB or config file

### 8. Item Count Configuration

Currently hardcoded to 500 items. Make configurable:

- Settings panel: number input for max items
- Rust: read config value, pass to `purge_oldest()`
- Persist in DB as a `settings` table

### 9. Drag-to-Reorder Pinned Items

Allow manual ordering of pinned items:

- `@dnd-kit` or simple drag API
- New DB column: `sort_order INTEGER`
- Only affects pinned items (they sort first)

### 10. Export / Import History

Backup and restore clipboard history:

- Rust command: `export_history()` → writes JSON to file
- Rus: `import_history(path)` → bulk insert from JSON
- Settings panel: export/import buttons

---

## Low Priority / Nice-to-Have

### 11. Dark/Light Mode Toggle

Follow system theme by default, allow manual toggle:

- Use `prefers-color-scheme` media query
- Tailwind `dark:` variants
- Settings: "System", "Dark", "Light" selector
- Tauri: `window.set_theme()`

### 12. App Name Detection

Detect which app the user copied from:

- macOS: use Accessibility API or `NSWorkspace` to get frontmost app name
- Store `app_name` in DB (already in schema)
- Frontend: show app icon + name next to preview

### 13. Snippet / Quick Paste Categories

Tag or categorize frequently used items (email, address, code snippets):

- DB: add `category TEXT` column
- Frontend: filter sidebar or tabs
- Quick access to common snippets

### 14. iCloud Sync / Multi-device

Sync clipboard history across Macs:

- Store DB in iCloud Drive directory
- Handle conflicts (last-write-wins or CRDT)
- Optional: toggle in settings

### 15. Fuzzy Search

Replace simple `LIKE` search with fuzzy matching:

- Frontend: `fuse.js` library for client-side fuzzy matching
- Or Rust: implement Levenshtein distance scoring
- Show match scores or highlight matched characters

---

## Polish / UX

| Item                      | Description                                                     |
| ------------------------- | --------------------------------------------------------------- |
| Empty state animation     | Subtle pulse or slide when no items exist                       |
| Scroll to top on new item | Auto-scroll virtualizer when new clipboard entry arrives        |
| Drag window to move       | Frameless window needs custom drag region (title bar area)      |
| Window resize             | Allow resize from corners, store last size                      |
| Tooltip on hover          | Show full content tooltip on hover when preview is truncated    |
| Keyboard shortcut hints   | Small labels showing shortcuts (e.g. "↵ to copy, Esc to close") |
| Sound effects             | Optional subtle click/pop sound on copy                         |
| Accessibility             | `aria-labels`, screen reader support, keyboard-only navigation  |
| Right-click menu          | Context menu on items: Copy, Paste, Pin, Delete                 |
| Item details view         | Double-click to expand and see full content                     |
