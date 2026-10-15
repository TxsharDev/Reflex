# Reflex

A command palette for Windows. Press **Alt+Space** and type: apps, files, clipboard history, math, and system commands in one place.

## Features

- **Apps**: everything in the Start menu, including Store apps, plus desktop shortcuts. Fuzzy matched and ranked by how often you launch them.
- **Files**: your home folder and every drive, indexed in the background. Type `.pdf` to search by extension, or several words to match across the path.
- **Clipboard**: text you copy during the session. Enter pastes into the window you came from. Content that password managers mark as private is never recorded.
- **Calculator**: `256 * 1024`, `2 ^ 10`, `0xFF`, `255 in hex`, `212f to c`, `2gb in mb`, `5 km to mi`, `3 lb to kg`.
- **Commands**: sleep, lock, restart, shut down, empty Recycle Bin, copy IP address, open terminal, screenshot, quit. Restart and shut down ask for a second Enter.

## Keys

| Key | Action |
| --- | --- |
| Alt+Space | Open / close |
| Up / Down | Move selection |
| Tab / Shift+Tab | Cycle filters (all, apps, files, clips, actions) |
| Enter | Open, paste, copy, or run |
| Ctrl+Enter | Run as admin (apps), show in folder (files), copy without pasting (clips) |
| Esc | Close and return focus |

Drag any empty part of the palette to move it. Clicking another window closes it.

## Build

Requires Windows 10 or 11 and a Rust toolchain.

```bash
git clone https://github.com/TxsharDev/Reflex
cd Reflex
cargo build --release
target\release\reflex.exe
```

To start it with Windows, put a shortcut to `reflex.exe` in `shell:startup`.

## Config

`%APPDATA%\reflex\config.json` is created on first run:

- `max_results`: rows shown at once (default 8)
- `index_dirs`: folders to index. Drive roots like `D:\` are indexed less deeply.

Launch counts are kept in `%APPDATA%\reflex\launch_counts.json`. Clipboard history stays in memory and is gone when Reflex exits. Nothing is sent over the network.

## Notes

- If another tool already uses Alt+Space (PowerToys Run, for example), the palette's status line says so.
- Discord may show its game overlay over the palette because Reflex renders with OpenGL. Turn it off in Discord under Settings > Game Overlay.

## License

MIT
