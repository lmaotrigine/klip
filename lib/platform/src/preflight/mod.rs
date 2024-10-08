#[cfg_attr(not(windows), path = "other.rs")]
#[cfg_attr(windows, path = "windows.rs")]
mod r#impl;
pub use r#impl::preflight;
