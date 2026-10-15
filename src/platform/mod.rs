#[cfg(not(windows))]
compile_error!("Reflex targets Windows only.");

mod windows;

pub use self::windows::*;
