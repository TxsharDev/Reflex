# Reflex

**Faster than thinking.**

An instant PC runtime for Windows. One hotkey opens everything - file search, app launcher, clipboard history, calculator, system commands. GPU-rendered, sub-3ms response, single binary.

Windows has no Raycast. No Alfred. No command palette for the OS. Reflex fills that gap.

## What it does

- **Command Palette** - Alt+Space, type anything. Files, apps, actions, math. Results in <3ms.
- **File Search** - Indexes your entire filesystem. Sub-10ms search across millions of files.
- **App Launcher** - Every installed app, ranked by how often you use them.
- **Clipboard History** - Never lose a copy again. Search and paste any previous clip.
- **Calculator** - Type math inline. Unit conversion. Hex/binary. Currency.
- **Quick Actions** - Sleep, shutdown, kill process, open terminal, screenshot. One command.

## What makes it different

- ~5MB binary. No Electron. No Node. No Python.
- <3ms to appear on screen. GPU-rendered UI.
- <20MB memory when idle.
- Zero cloud. Zero telemetry. Zero accounts.
- Keyboard-first. Mouse-optional.
- Single binary. No installer.

## Planned

- Window management and snap zones
- Workspace save/restore (close everything, reopen it all later)
- Plugin SDK (Rust/WASM)
- Context-aware results (different actions per active app)
- Local semantic search (optional, no cloud)

## Tech

Rust. wgpu for rendering. windows-rs for OS integration. Custom fuzzy matching and file indexer.

## Status

In development.

---

Built by [Tushar Sharma](https://github.com/TxsharDev)
