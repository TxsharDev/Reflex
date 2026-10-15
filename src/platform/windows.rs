//! Win32 glue: global hotkey, window placement and focus, app discovery,
//! clipboard privacy flags, and shell/system commands.

use std::collections::HashSet;
use std::ffi::c_void;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::mpsc;

use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::Foundation::{
    CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HGLOBAL, HWND, LUID, POINT, RECT,
};
use windows::Win32::Graphics::Dwm::{
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
};
use windows::Win32::Security::{
    AdjustTokenPrivileges, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED,
    SE_SHUTDOWN_NAME, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES,
};
use windows::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree, IBindCtx,
};
use windows::Win32::System::DataExchange::{
    CloseClipboard, GetClipboardData, GetClipboardSequenceNumber, IsClipboardFormatAvailable,
    OpenClipboard, RegisterClipboardFormatW,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use windows::Win32::System::Power::SetSuspendState;
use windows::Win32::System::Shutdown::LockWorkStation;
use windows::Win32::System::Threading::{CreateMutexW, GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, MOD_ALT,
    MOD_NOREPEAT, RegisterHotKey, SendInput, VIRTUAL_KEY, VK_CONTROL, VK_SPACE, VK_V,
};
use windows::Win32::UI::Shell::{
    BHID_EnumItems, FOLDERID_AppsFolder, IEnumShellItems, IShellItem, KF_FLAG_DEFAULT,
    SHERB_NOSOUND, SHEmptyRecycleBinW, SHGetKnownFolderItem, SIGDN, SIGDN_NORMALDISPLAY,
    SIGDN_PARENTRELATIVEPARSING, ShellExecuteW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetMessageW, GetShellWindow, GetWindowRect, HWND_TOPMOST,
    IsWindow, MSG, SW_SHOWNORMAL, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetForegroundWindow,
    SetWindowPos, WM_HOTKEY,
};
use windows::core::{HSTRING, PCWSTR, w};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Returns false if another Reflex process is already running.
pub fn claim_single_instance() -> bool {
    // The mutex handle is intentionally never closed; it lives as long as the process.
    unsafe {
        CreateMutexW(None, true, w!("Local\\Reflex.SingleInstance")).is_ok()
            && GetLastError() != ERROR_ALREADY_EXISTS
    }
}

/// The palette's native window.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Window(isize);

impl Window {
    pub fn from_handle(source: &impl HasWindowHandle) -> Option<Self> {
        match source.window_handle().ok()?.as_raw() {
            RawWindowHandle::Win32(handle) => Some(Self(handle.hwnd.get())),
            _ => None,
        }
    }

    fn hwnd(self) -> HWND {
        HWND(self.0 as *mut c_void)
    }

    /// Windows 11 rounded corners. Older versions ignore the attribute.
    pub fn round_corners(self) {
        let preference = DWMWCP_ROUND;
        unsafe {
            let _ = DwmSetWindowAttribute(
                self.hwnd(),
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &preference as *const _ as *const c_void,
                size_of_val(&preference) as u32,
            );
        }
    }

    /// Moves to the monitor under the cursor (centered, top edge at a fifth of
    /// the work area) and takes keyboard focus. Returns false if Windows
    /// refused the focus change.
    pub fn present(self) -> bool {
        unsafe {
            let mut cursor = POINT::default();
            let _ = GetCursorPos(&mut cursor);
            let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
            let mut info = MONITORINFO {
                cbSize: size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let mut rect = RECT::default();
            if GetMonitorInfoW(monitor, &mut info).as_bool()
                && GetWindowRect(self.hwnd(), &mut rect).is_ok()
            {
                let work = info.rcWork;
                let width = rect.right - rect.left;
                let x = work.left + (work.right - work.left - width) / 2;
                let y = work.top + (work.bottom - work.top) / 5;
                let _ = SetWindowPos(self.hwnd(), Some(HWND_TOPMOST), x, y, 0, 0, SWP_NOSIZE);
            }
            SetForegroundWindow(self.hwnd()).as_bool()
        }
    }

    /// Moves the window off every monitor. It stays alive so the event loop
    /// keeps running and the next show is instant.
    pub fn park(self) {
        unsafe {
            let _ = SetWindowPos(
                self.hwnd(),
                None,
                -32000,
                -32000,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }
}

/// Handle of the window that currently has focus.
pub fn foreground() -> isize {
    unsafe { GetForegroundWindow().0 as isize }
}

/// Gives focus back to `handle`, or to the desktop if that window is gone.
pub fn restore_focus(handle: isize) {
    unsafe {
        let hwnd = HWND(handle as *mut c_void);
        if handle == 0 || !IsWindow(Some(hwnd)).as_bool() || !SetForegroundWindow(hwnd).as_bool() {
            let _ = SetForegroundWindow(GetShellWindow());
        }
    }
}

/// Registers Alt+Space on a dedicated message thread. Each press records the
/// window that had focus (unless it was the palette), raises `pressed`, and
/// wakes the UI. Returns false if another program already owns the shortcut.
pub fn register_hotkey(
    ctx: egui::Context,
    palette: Window,
    pressed: Arc<AtomicBool>,
    previous: Arc<AtomicIsize>,
) -> bool {
    const HOTKEY_ID: i32 = 1;
    let (registered, result) = mpsc::channel();

    std::thread::spawn(move || unsafe {
        let ok = RegisterHotKey(None, HOTKEY_ID, MOD_ALT | MOD_NOREPEAT, VK_SPACE.0 as u32).is_ok();
        let _ = registered.send(ok);
        if !ok {
            return;
        }
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_HOTKEY && msg.wParam.0 == HOTKEY_ID as usize {
                let focused = foreground();
                if focused != palette.0 {
                    previous.store(focused, Ordering::SeqCst);
                }
                pressed.store(true, Ordering::SeqCst);
                ctx.request_repaint();
            }
        }
    });

    result.recv().unwrap_or(false)
}

/// Types Ctrl+V into whichever window has focus.
pub fn send_paste() {
    let key = |vk: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                dwFlags: flags,
                ..Default::default()
            },
        },
    };
    let none = KEYBD_EVENT_FLAGS(0);
    let inputs = [
        key(VK_CONTROL, none),
        key(VK_V, none),
        key(VK_V, KEYEVENTF_KEYUP),
        key(VK_CONTROL, KEYEVENTF_KEYUP),
    ];
    unsafe {
        SendInput(&inputs, size_of::<INPUT>() as i32);
    }
}

/// Changes every time anything is copied.
pub fn clipboard_sequence() -> u32 {
    unsafe { GetClipboardSequenceNumber() }
}

/// True when the program that owns the clipboard asked history tools to skip
/// it. Password managers mark copied secrets this way.
pub fn clipboard_is_private() -> bool {
    unsafe {
        let exclude = RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing"));
        if IsClipboardFormatAvailable(exclude).is_ok() {
            return true;
        }
        let history = RegisterClipboardFormatW(w!("CanIncludeInClipboardHistory"));
        if IsClipboardFormatAvailable(history).is_err() {
            return false;
        }
        if OpenClipboard(None).is_err() {
            return true;
        }
        let mut allowed = true;
        if let Ok(data) = GetClipboardData(history) {
            let memory = HGLOBAL(data.0);
            let flag = GlobalLock(memory) as *const u32;
            if !flag.is_null() {
                allowed = *flag != 0;
                let _ = GlobalUnlock(memory);
            }
        }
        let _ = CloseClipboard();
        !allowed
    }
}

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    pub path: PathBuf,
    pub launch_count: u32,
}

/// Everything in the Start menu's app list (desktop and Store apps), plus
/// shortcuts on the desktop.
pub fn discover_apps() -> Vec<AppEntry> {
    let mut seen = HashSet::new();
    // Shell enumeration wants a single-threaded apartment of its own.
    let mut apps = std::thread::spawn(installed_apps)
        .join()
        .unwrap_or_default();
    apps.retain(|app| seen.insert(app.name.to_lowercase()));

    if let Some(desktop) = dirs::desktop_dir()
        && let Ok(entries) = std::fs::read_dir(desktop)
    {
        for path in entries.flatten().map(|e| e.path()) {
            let is_shortcut = path.extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("lnk") || ext.eq_ignore_ascii_case("exe")
            });
            let Some(name) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) else {
                continue;
            };
            if is_shortcut && seen.insert(name.to_lowercase()) {
                apps.push(AppEntry {
                    name,
                    path,
                    launch_count: 0,
                });
            }
        }
    }

    apps.sort_by_key(|a| a.name.to_lowercase());
    apps
}

/// Walks `shell:AppsFolder`. Each app launches through its AppsFolder path,
/// which works for Store apps that have no executable to point at.
fn installed_apps() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let Ok(folder) =
            SHGetKnownFolderItem::<IShellItem>(&FOLDERID_AppsFolder, KF_FLAG_DEFAULT, None)
        else {
            return apps;
        };
        let Ok(items) =
            folder.BindToHandler::<_, IEnumShellItems>(None::<&IBindCtx>, &BHID_EnumItems)
        else {
            return apps;
        };
        let mut batch = [None];
        while items.Next(&mut batch, None).is_ok() {
            let Some(item) = batch[0].take() else { break };
            let (Some(name), Some(id)) = (
                display_name(&item, SIGDN_NORMALDISPLAY),
                display_name(&item, SIGDN_PARENTRELATIVEPARSING),
            ) else {
                continue;
            };
            if !name.to_lowercase().contains("uninstall") {
                let path = PathBuf::from(format!(r"shell:AppsFolder\{id}"));
                apps.push(AppEntry {
                    name,
                    path,
                    launch_count: 0,
                });
            }
        }
    }
    apps
}

unsafe fn display_name(item: &IShellItem, form: SIGDN) -> Option<String> {
    unsafe {
        let raw = item.GetDisplayName(form).ok()?;
        let text = raw.to_string().ok();
        CoTaskMemFree(Some(raw.0 as *const c_void));
        text.filter(|t| !t.is_empty())
    }
}

/// Opens a file, folder, shortcut, or URI through the shell.
/// `elevate` shows the UAC prompt to run it as administrator.
pub fn shell_open(target: &Path, elevate: bool) {
    let verb = if elevate { w!("runas") } else { PCWSTR::null() };
    let file = HSTRING::from(target.as_os_str());
    unsafe {
        ShellExecuteW(
            None,
            verb,
            &file,
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
    }
}

/// Opens Explorer with `path` selected.
pub fn reveal_in_explorer(path: &Path) {
    let _ = Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{}\"", path.display()))
        .spawn();
}

pub fn open_terminal() {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("C:\\"));
    let spawned = Command::new("wt.exe").current_dir(&home).spawn();
    if spawned.is_err() {
        let _ = Command::new("cmd.exe").current_dir(&home).spawn();
    }
}

pub fn sleep() {
    // SetSuspendState needs the shutdown privilege, which is held but not
    // enabled by default. It blocks until the machine wakes, so run it off
    // the UI thread.
    std::thread::spawn(|| unsafe {
        enable_shutdown_privilege();
        let _ = SetSuspendState(false, false, false);
    });
}

pub fn lock() {
    unsafe {
        let _ = LockWorkStation();
    }
}

pub fn shutdown(restart: bool) {
    let mode = if restart { "/r" } else { "/s" };
    let _ = Command::new("shutdown.exe")
        .args([mode, "/t", "0"])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
}

/// Windows shows its own confirmation dialog before emptying.
pub fn empty_recycle_bin() {
    unsafe {
        let _ = SHEmptyRecycleBinW(None, PCWSTR::null(), SHERB_NOSOUND);
    }
}

/// Address of the interface used for outbound traffic. Nothing is sent: a
/// UDP connect only picks a route.
pub fn local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}

unsafe fn enable_shutdown_privilege() {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES, &mut token).is_err() {
            return;
        }
        let mut luid = LUID::default();
        if LookupPrivilegeValueW(PCWSTR::null(), SE_SHUTDOWN_NAME, &mut luid).is_ok() {
            let privileges = TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                Privileges: [LUID_AND_ATTRIBUTES {
                    Luid: luid,
                    Attributes: SE_PRIVILEGE_ENABLED,
                }],
            };
            let _ = AdjustTokenPrivileges(token, false, Some(&privileges), 0, None, None);
        }
        let _ = CloseHandle(token);
    }
}
