//! Windows platform integration - hotkeys, app discovery, window management.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::path::PathBuf;

// -- Window rounding (DWM) --

/// Apply rounded corners to the window via DWM on Windows 11.
pub fn apply_rounded_corners() {
    use windows::Win32::UI::WindowsAndMessaging::*;

    unsafe {
        let title: Vec<u16> = "Reflex\0".encode_utf16().collect();
        let Ok(hwnd) = FindWindowW(None, windows::core::PCWSTR(title.as_ptr())) else { return };

        // DWM rounded corners (Windows 11+)
        let preference: u32 = 2; // DWMWCP_ROUND
        let _ = windows::Win32::Graphics::Dwm::DwmSetWindowAttribute(
            hwnd,
            windows::Win32::Graphics::Dwm::DWMWINDOWATTRIBUTE(33),
            &preference as *const u32 as *const std::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        );

        // Tool window style - no taskbar entry
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_TOOLWINDOW.0 as i32);
    }
}

// -- Global hotkey --

const HOTKEY_ID: i32 = 1;

/// Registers Alt+Space as a global hotkey. Spawns a thread that listens
/// for the hotkey and sets the atomic flag when triggered.
pub fn register_hotkey(triggered: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        use windows::Win32::UI::Input::KeyboardAndMouse::*;
        use windows::Win32::UI::WindowsAndMessaging::*;

        unsafe {
            let _ = RegisterHotKey(
                None,
                HOTKEY_ID,
                MOD_ALT | MOD_NOREPEAT,
                0x20, // VK_SPACE
            );

            let mut msg = MSG::default();
            loop {
                if GetMessageW(&mut msg, None, 0, 0).as_bool()
                    && msg.message == WM_HOTKEY && msg.wParam.0 as i32 == HOTKEY_ID
                {
                    triggered.store(true, Ordering::SeqCst);
                }
            }
        }
    });
}

// -- App discovery --

/// Find all installed applications by scanning common locations.
pub fn discover_apps() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Start Menu (current user) - most reliable
    if let Ok(appdata) = std::env::var("APPDATA") {
        let start_menu = PathBuf::from(appdata)
            .join("Microsoft").join("Windows").join("Start Menu").join("Programs");
        scan_shortcuts(&start_menu, &mut apps, &mut seen, 0);
    }

    // ProgramData start menu
    let prog_data = PathBuf::from("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs");
    scan_shortcuts(&prog_data, &mut apps, &mut seen, 0);

    // Desktop shortcuts (non-recursive)
    if let Some(desktop) = dirs::desktop_dir() {
        scan_shortcuts_flat(&desktop, &mut apps, &mut seen);
    }

    apps.sort_by_key(|a| a.name.to_lowercase());
    apps
}

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    pub path: PathBuf,
    pub launch_count: u32,
}

fn scan_shortcuts(dir: &PathBuf, apps: &mut Vec<AppEntry>, seen: &mut std::collections::HashSet<String>, depth: usize) {
    if depth > 3 { return; } // Don't recurse too deep
    let Ok(entries) = std::fs::read_dir(dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_shortcuts(&path, apps, seen, depth + 1);
        } else {
            add_shortcut(&path, apps, seen);
        }
    }
}

fn scan_shortcuts_flat(dir: &PathBuf, apps: &mut Vec<AppEntry>, seen: &mut std::collections::HashSet<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            add_shortcut(&path, apps, seen);
        }
    }
}

fn add_shortcut(path: &std::path::Path, apps: &mut Vec<AppEntry>, seen: &mut std::collections::HashSet<String>) {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        if ext == "lnk" || ext == "exe" {
            let name = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            if !name.is_empty() && !seen.contains(&name.to_lowercase()) {
                seen.insert(name.to_lowercase());
                apps.push(AppEntry {
                    name,
                    path: path.to_path_buf(),
                    launch_count: 0,
                });
            }
        }
    }
}

/// Launch an application.
pub fn launch_app(path: &PathBuf, as_admin: bool) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    if as_admin {
        // ShellExecuteW with "runas" verb
        let path_str = path.to_string_lossy().to_string();
        let _ = std::process::Command::new("powershell")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["-NoProfile", "-Command", &format!("Start-Process '{}' -Verb RunAs", path_str)])
            .spawn();
    } else {
        let _ = open::that(path);
    }
}

// -- Process management --

#[allow(dead_code)]
pub fn kill_process(name: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let output = std::process::Command::new("taskkill")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["/F", "/IM", name])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

// -- System actions --

pub fn system_action(action: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    match action {
        "sleep" => {
            let _ = std::process::Command::new("rundll32.exe")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["powrprof.dll,SetSuspendState", "0,1,0"])
                .spawn();
        }
        "shutdown" => {
            let _ = std::process::Command::new("shutdown")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/s", "/t", "0"])
                .spawn();
        }
        "restart" => {
            let _ = std::process::Command::new("shutdown")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/r", "/t", "0"])
                .spawn();
        }
        "lock" => {
            let _ = std::process::Command::new("rundll32.exe")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["user32.dll,LockWorkStation"])
                .spawn();
        }
        "emptybin" => {
            let _ = std::process::Command::new("powershell")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["-NoProfile", "-Command", "Clear-RecycleBin -Force"])
                .spawn();
        }
        _ => return Err(format!("Unknown action: {}", action)),
    }
    Ok(())
}

/// Get local IP address.
pub fn get_ip() -> String {
    use std::net::UdpSocket;
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("8.8.8.8:80")?;
            s.local_addr()
        })
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}
