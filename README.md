# Reflex

**Faster than thinking.**

The command palette Windows never built. One hotkey opens everything - apps, files, clipboard, calculator, system commands. GPU-rendered, instant results, single binary.

## What it does

**Alt+Space** and start typing.

- **Apps** - every installed app, fuzzy matched, ranked by usage. Enter to launch, Ctrl+Enter for admin.
- **Files** - searches your filesystem in the background. Results in milliseconds. Ctrl+Enter opens containing folder.
- **Clipboard** - never lose a copy again. Every clip stored, searchable, one Enter to paste.
- **Calculator** - type math inline: `256 * 1024`, `0xFF`, `100 f to c`, `32 gb in mb`.
- **System** - sleep, shutdown, restart, lock, empty recycle bin, get IP, open terminal, screenshot.
- **Filters** - Tab cycles through All/Apps/Files/Clipboard/Actions.

## Why this exists

Windows in 2026 has no command palette. Start Menu searches the web. PowerToys Run is slow. Flow Launcher is Electron. Everything is file-only. Ditto is clipboard-only. Raycast and Alfred are Mac-only.

Reflex is all of them in one ~8MB binary. No installer. No Electron. No cloud. No accounts.

## Design

- Borderless floating window with OS-level rounded corners
- Dark theme, custom-painted result rows
- Color-coded results (green=app, blue=file, yellow=calc, purple=clip, red=action)
- Keyboard hints on selected results
- No taskbar entry - appears and disappears like Spotlight
- Dynamic sizing - grows with results, shrinks when empty

## Tech

Rust + egui/eframe (glow) + windows-rs. Custom file indexer, fuzzy matching, clipboard watcher, global hotkey via Win32 RegisterHotKey, DWM rounded corners.

## Status

In development. Core features working.

## Planned

- Window management and snap zones
- Workspace save/restore
- Plugin SDK (Rust/WASM)
- Context-aware results per active app
- Customizable hotkey
- Light theme

---

Built by [Tushar Sharma](https://github.com/TxsharDev)
