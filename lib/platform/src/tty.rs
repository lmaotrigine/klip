#[cfg(windows)]
#[inline]
#[must_use]
pub fn isatty(stderr: bool) -> bool {
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
    };

    unsafe {
        let handle = GetStdHandle(if stderr {
            STD_ERROR_HANDLE
        } else {
            STD_OUTPUT_HANDLE
        });
        let mut out = 0;
        GetConsoleMode(handle, &mut out) != 0
    }
}

#[cfg(unix)]
#[inline]
#[must_use]
pub fn isatty(stderr: bool) -> bool {
    unsafe {
        libc::isatty(if stderr {
            libc::STDERR_FILENO
        } else {
            libc::STDOUT_FILENO
        }) != 0
    }
}

#[cfg(not(any(windows, unix)))]
#[inline]
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn isatty(_stderr: bool) -> bool {
    false
}
