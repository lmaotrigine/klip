use crypto_common::erase::{Erase, EraseOnDrop};

#[cfg_attr(not(any(windows, unix)), path = "fallback.rs")]
#[cfg_attr(windows, path = "windows.rs")]
#[cfg_attr(unix, path = "unix.rs")]
mod r#impl;

/// A newtype wrapper for [`String`] that is zeroed out on drop.
struct Password(String);

impl Password {
    const fn new() -> Self {
        Self(String::new())
    }

    fn into_inner(mut self) -> String {
        core::mem::take(&mut self.0)
    }
}

impl Erase for Password {
    fn erase(&mut self) {
        // SAFETY: all bytes are set to 0, which is valid UTF-8.
        for ch in unsafe { self.0.as_bytes_mut() } {
            ch.erase();
        }
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}

impl EraseOnDrop for Password {}

fn fix_line(mut line: String) -> std::io::Result<String> {
    if !line.ends_with('\n') {
        return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
    }
    line.pop();
    if line.ends_with('\r') {
        line.pop();
    }
    line = match line.find('\u{0015}') {
        Some(c_u) => line[c_u + 1..].to_string(),
        None => line,
    };
    Ok(line)
}

/// Reads the password through the attached TTY, or stdin if there is no TTY.
///
/// # Errors
///
/// This function will return an error if the password could not be read. This
/// error will be propagated from the underlying I/O operation.
pub fn get() -> std::io::Result<String> {
    if crate::tty::isatty(false) {
        r#impl::print_tty("Password: ")?;
        r#impl::read_password()
    } else {
        let mut password = Password::new();
        std::io::stdin().read_line(&mut password.0)?;
        fix_line(password.into_inner())
    }
}
