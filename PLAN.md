# Reflex - Feature Plan

## The Problem

Windows in 2026 is embarrassingly slow to interact with. Not the OS itself - the interaction layer. The gap between "I want to do something" and "it's done" is filled with:

- Start menu that takes 300ms to open and searches the web instead of your files
- Alt-tab that shows tiny thumbnails you can't read
- No command palette (every modern app has one, the OS doesn't)
- Clipboard that forgets everything the moment you copy something new
- File search that's either slow (Windows Search) or ugly (Everything)
- Window management that requires dragging pixels around manually
- No way to save and restore your workspace
- Settings scattered across 15 different control panels
- No quick math, no unit conversion, no instant actions without opening an app

Every one of these has a separate tool. PowerToys, Flow Launcher, Ditto, Everything, FancyZones, ShareX. Six apps to do what one app should do.

## The Solution

Reflex is a single native runtime that makes Windows feel instant. One hotkey. Everything you need.

## Core Philosophy

- 3ms or less to appear on screen
- Keyboard-first, mouse-optional
- Zero cloud, zero telemetry, zero accounts
- Sub-20MB memory when idle
- Single binary, no installer required
- GPU-rendered UI for buttery animations
- Index once, search forever
- Plugin system for extensibility
- Looks like it belongs in 2026, not 2006

---

## V1 Features (MVP - ship this)

### 1. Command Palette (the foundation)
- Global hotkey (Alt+Space default, configurable)
- Appears in <3ms, centered on active monitor
- Type to search everything - files, apps, actions, clipboard, calculator
- Fuzzy matching with smart ranking
- Recent/frequent items prioritized
- Escape or click outside to dismiss
- GPU-rendered with smooth fade in/out

### 2. App Launcher
- Index all installed apps (Start Menu, Desktop, PATH, custom dirs)
- Launch with Enter
- "Run as admin" with Ctrl+Enter
- Pin favorites
- Track launch frequency for ranking

### 3. File Search
- Background indexer (like Everything but built-in)
- Index all drives on first run, incremental updates via filesystem watcher
- Search by name, extension, path
- Preview file info (size, modified date, type)
- Open file, open containing folder, copy path
- Sub-10ms search across millions of files

### 4. Clipboard History
- Store last 100+ clips (text, images, files)
- Search through clipboard history
- Paste any previous clip with one action
- Pin important clips permanently
- Clear history with one command
- Sensitive content detection (don't store passwords from password managers)

### 5. Calculator + Unit Converter
- Type math directly: "256 * 1024" shows result inline
- Unit conversion: "5090 watts to kilowatts", "32gb in mb", "100f to c"
- Currency conversion (offline rates, updated periodically)
- Hex/binary/decimal: "0xFF in decimal", "255 in hex"
- Result auto-copies on Enter

### 6. Quick Actions
- "sleep" / "shutdown" / "restart" / "lock"
- "empty recycle bin"
- "ip address"
- "kill [process name]"
- "open terminal here"
- "screenshot" / "screen record"
- Extensible via plugins later

---

## V2 Features (after V1 ships and gets feedback)

### 7. Window Manager
- Snap zones (like FancyZones but faster)
- Keyboard shortcuts for window placement
- "Move window to monitor 2"
- Save window layouts as named workspaces

### 8. Workspace Save/Restore (the killer feature)
- Save current state: which apps are open, window positions, active tabs
- Restore a workspace by name
- "Work" workspace: VS Code + Terminal + Browser + Discord
- "Gaming" workspace: Game + Discord + Pane
- Per-workspace GPU power profiles

### 9. System Commands
- Volume control: "volume 50%", "mute"
- Bluetooth: "connect airpods"
- WiFi: "connect to [network]"
- Display: "brightness 80%"
- Deep Settings shortcuts

### 10. File Actions
- Quick rename
- Batch operations: "rename all .jpg to .png"
- Quick compress/extract
- Move to common locations
- Hash file

---

## V3 Features (if it takes off)

### 11. Plugin SDK
- Rust or WASM plugins
- Custom search providers
- Custom actions
- Custom UI panels
- Community plugin repository

### 12. Context Awareness
- Different results based on active app
- In VS Code: show recent projects, terminals
- In browser: show bookmarks, history
- In file explorer: show file actions

### 13. Automation / Macros
- Record and replay action sequences
- Scheduled actions
- Trigger chains: "when I open Photoshop, move to monitor 2 and set GPU to performance"

### 14. AI-Powered Search (optional, local)
- Semantic file search using local embeddings
- "that document about the project budget" finds it without exact filename
- Completely local, no cloud
- Opt-in, not default

---

## Architecture

```
reflex/
  src/
    main.rs           # Entry point, hotkey listener, window management
    ui/               # GPU-rendered UI (wgpu + custom widgets)
      palette.rs      # Command palette rendering
      theme.rs        # Colors, fonts, animations
      widgets.rs      # Input field, result list, preview
    search/           # Search engine
      index.rs        # File system indexer
      apps.rs         # App discovery and indexing
      fuzzy.rs        # Fuzzy matching algorithm
      ranker.rs       # Result ranking (frequency, recency, relevance)
    clipboard/        # Clipboard manager
      history.rs      # Clip storage and retrieval
      watcher.rs      # Clipboard monitoring
    actions/          # Built-in actions
      system.rs       # Sleep, shutdown, lock, etc.
      calculator.rs   # Math and unit conversion
      files.rs        # File operations
    config/           # Settings
      mod.rs          # Config file (JSON)
      hotkeys.rs      # Hotkey configuration
    platform/         # OS-specific
      windows.rs      # Win32 hotkey, window management, app discovery
      linux.rs        # X11/Wayland equivalents
```

## Tech Stack

- Rust
- wgpu for GPU-rendered UI (not egui - need pixel-perfect control for a launcher)
- windows-rs for Win32 APIs (hotkeys, clipboard, window management)
- notify crate for filesystem watching
- Custom fuzzy matching (nucleo or similar)
- serde + JSON for config
- Single binary, no runtime

## Competition

| Tool | What it does | What's wrong |
|------|-------------|--------------|
| Windows Start Menu | Search + launch | 300ms+ latency, web results pollution, ugly |
| PowerToys Run | Launcher | Slow, limited, no clipboard, no file indexing |
| Flow Launcher | Launcher + plugins | Electron-tier, heavy, plugin quality varies |
| Everything | File search | Fast but ugly, search only, no actions |
| Ditto | Clipboard | Standalone only, old UI |
| Raycast | Everything | Mac only |
| Alfred | Everything | Mac only |
| Wox | Launcher | Abandoned |
| Listary | File search + launcher | Paid, dated UI |

Reflex = Everything + Ditto + PowerToys Run + calculator + quick actions, in one fast binary.

## Success Metrics

- <3ms palette open time
- <10ms search results for 1M+ files
- <20MB idle memory
- <5MB binary size
- Zero dropped frames in animations
- 10,000+ GitHub stars within 6 months (ambitious but achievable if it's genuinely fast)

## Name

Reflex - faster than thinking.
