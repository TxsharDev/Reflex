# Reflex

**Faster than thinking.**

The command palette Windows never built. One hotkey opens everything - apps, files, clipboard, calculator, system commands. GPU-rendered, instant results, single binary.

## What it does

**Alt+Space** and start typing.

- **Apps** - every installed app, fuzzy matched (nucleo), ranked by usage. Enter to launch, Ctrl+Enter for admin.
- **Files** - searches your filesystem in the background. Two-phase indexer: user dirs first, then drives.
- **Clipboard** - never lose a copy again. 200 clips stored, searchable, one Enter to paste.
- **Calculator** - type math inline: `256 * 1024`, `0xFF`, `100 f to c`, `32 gb in mb`.
- **System** - sleep, shutdown, restart, lock, empty recycle bin, get IP, open terminal, screenshot, quit.
- **Filters** - Tab cycles through All/Apps/Files/Clipboard/Actions.

## Why this exists

Windows in 2026 has no command palette. Start Menu searches the web. PowerToys Run is slow. Flow Launcher is Electron. Everything is file-only. Ditto is clipboard-only. Raycast and Alfred are Mac-only.

Reflex is all of them in one ~8MB binary. No installer. No Electron. No cloud. No accounts.

## Design

- Borderless floating window with OS-level rounded corners (DWM, Win11)
- Dark theme, custom-painted result rows
- Color-coded results (green=app, blue=file, yellow=calc, purple=clip, red=action)
- Keyboard hints on selected results
- No taskbar entry - appears and disappears like Spotlight
- 120ms fade in/out - never feels abrupt
- Dynamic sizing - grows with results, shrinks when empty

## Quick start

```bash
# Clone and build
git clone https://github.com/TxsharDev/Reflex
cd Reflex
cargo build --release

# Run (single binary, no installer)
target\release\reflex.exe
```

## Tech

Rust + egui/eframe (glow) + windows-rs. Nucleo fuzzy matching, two-phase file indexer, clipboard watcher, global hotkey via Win32 RegisterHotKey, DWM rounded corners.

## Config

`%APPDATA%\reflex\config.json` - max results, index dirs, theme.
`%APPDATA%\reflex\launch_counts.json` - app frequency ranking (persists across restarts).

## Quit

Type `quit` in the palette and hit Enter. No tray icon, no config to kill the hotkey - just type quit.

## Status

v2.1 - core features solid, polished for daily use.

## Planned

- Window management and snap zones
- Workspace save/restore
- Plugin SDK (Rust/WASM)
- Context-aware results per active app
- Incremental file watching (notify crate)
- Image clipboard support
- Customizable hotkey
- Light theme

---

Built by [Tushar Sharma](https://github.com/TxsharDev)
