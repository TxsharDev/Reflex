# Changelog

All notable changes to Reflex will be documented here.

## [2.1.0] - 2026-08-21

### Added
- **Fade in/out animation** - 120ms alpha lerp, transparent window, no more jarring pop
- **Launch count persistence** - frequency ranking survives restarts (`launch_counts.json`)
- **Quit action** - type "quit" or "exit" in the palette to close Reflex
- **Unit tests** - 15 tests covering calculator parser (math, hex, binary, units) and nucleo fuzzy matching
- **CI** - GitHub Actions: `cargo test` + `cargo clippy` on every push (Windows)

### Changed
- **Fuzzy matching** - app search now uses nucleo (same engine as helix) instead of a local subsequence matcher
- **File indexer** - uses config `index_dirs` directly (was ignoring the param and re-deriving internally)
- **Unit conversion** - handles glued value+unit input: `212f to c`, `2gb in mb`
- **Repaint rate** - 50ms → 30ms for smoother animation

### Fixed
- `start_indexing` ignored its `dirs` parameter (underscore prefix)
- Duplicate fuzzy logic: local `fuzzy_contains` in main.rs shadowed the nucleo module
- `nucleo_score` was `u32` but had a redundant `as u32` cast
- Unit conversion missed 3-token inputs (value+unit glued together)

### Removed
- `image` crate dependency (unused, for future image clipboard support)
- Global `#![allow(dead_code)]` — replaced with targeted per-item attributes
- Local `fuzzy_contains` function in main.rs

## [2.0.0] - 2026-05-26

### Added
- **Command palette** - Alt+Space global hotkey, centered floating window, appears/disappears instantly
- **App launcher** - 294 apps discovered from Start Menu + Desktop, fuzzy search, launch frequency tracking
- **File search** - two-phase background indexer (user dirs first, then all drives), extension search (.rs, .py), multi-word search, smart directory skipping
- **Clipboard history** - monitors clipboard in real-time, searchable, paste any previous clip, auto-dedup
- **Calculator** - inline math (256*1024), hex (0xFF, 0b1111), unit conversion (temp, data, distance, weight), binary conversion
- **System actions** - sleep, shutdown, restart, lock, empty recycle bin, IP address (copies to clipboard), terminal, screenshot
- **Filter tabs** - All/Apps/Files/Clipboard/Actions with Tab to cycle, result count per filter
- **Keyboard navigation** - Arrow keys, Enter to execute, Ctrl+Enter for admin launch/open containing folder, Escape to dismiss, Tab to cycle filters
- **Empty state** - shows recent apps and clipboard entries when palette opens with no query
- **Custom-painted result rows** - color dots per type, type badges, keyboard action hints on selected row
- **OS-level rounded corners** via Windows 11 DWM API
- **No taskbar entry** - tool window style via WS_EX_TOOLWINDOW
- **No console window** - pure GUI app via windows_subsystem
- Dynamic window resizing to fit result count
- Window moves offscreen when hidden (keeps event loop alive for hotkey)
- Config persistence (JSON) for max results and index directories

### Changed
- Switched from wgpu to glow backend for compatibility
- Premium dark theme with refined color palette

### Fixed
- All clippy warnings resolved (zero warnings policy)
- TextEdit no longer swallows Enter key (return_key set to None)
- File indexer skips system directories, node_modules, .git, AppData, etc.
- Clipboard watcher uses collapsed if-let chains for cleaner control flow

## [1.0.0] - 2026-05-26

### Added
- Project initialized
- Feature plan with V1/V2/V3 roadmap
- Architecture designed
- MIT License
