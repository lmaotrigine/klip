#[cfg_attr(windows, path = "windows.rs")]
#[cfg_attr(unix, path = "unix.rs")]
#[cfg_attr(not(any(windows, unix)), path = "fallback.rs")]
mod r#impl;
pub use r#impl::home_dir;
