#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use self::windows::*;

#[cfg(not(target_os = "windows"))]
pub mod stub;

#[cfg(not(target_os = "windows"))]
pub use stub::*;
