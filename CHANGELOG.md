# Changelog

All notable changes to Reflex will be documented here.

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
