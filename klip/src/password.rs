use crypto_common::erase::{Erase, EraseOnDrop};

struct EraseableString(String);

impl EraseableString {
    pub(self) const fn new() -> Self {
        Self(String::new())
    }

    pub(self) fn into_inner(mut self) -> String {
        std::mem::take(&mut self.0)
    }
}

impl Erase for EraseableString {
    fn erase(&mut self) {
        for ch in unsafe { self.0.as_bytes_mut() } {
            ch.erase();
        }
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}

impl Drop for EraseableString {
    fn drop(&mut self) {
        self.erase();
    }
}

impl EraseOnDrop for EraseableString {}

impl core::ops::Deref for EraseableString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for EraseableString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn fix_line(mut line: String) -> std::io::Result<String> {
    if !line.ends_with('\n') {
        return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
    }
    line.pop();
    if line.ends_with('\r') {
        line.pop();
    }
    line = match line.rfind('\u{0015}') {
        Some(ctrl_u) => line[ctrl_u + 1..].to_owned(),
        None => line,
    };

    Ok(line)
}

#[cfg(not(any(windows, unix)))]
mod inner {
    use std::{
        fs::File,
        io::{self, BufRead},
    };
    pub fn read_password() -> io::Result<String> {
        let tty = File::open("/dev/tty")?;
        let mut reader = io::BufReader::new(tty);
        let mut password = super::EraseableString::new();
        reader.read_line(&mut password)?;
        super::fix_line(password.into_inner())
    }

    pub fn print_tty(prompt: &str) -> io::Result<()> {
        write!(io::stdout(), "{prompt}")?;
        io::stdout().flush()?;
        Ok(())
    }
}

#[cfg(unix)]
mod inner {
    use std::{
        fs::{File, OpenOptions},
        io::{self, BufRead, Write},
        mem::MaybeUninit,
        os::unix::io::AsRawFd,
    };

    struct Hidden {
        fd: i32,
        termios: libc::termios,
    }

    impl Hidden {
        fn new(fd: i32) -> io::Result<Self> {
            // we can't just `let original = term` because original gets modified by
            // `tcsetattr` as well. so 2 calls it is.
            macro_rules! tcgetattr {
                () => {{
                    let mut t = MaybeUninit::uninit();
                    if unsafe { libc::tcgetattr(fd, t.as_mut_ptr()) } != 0 {
                        return Err(io::Error::last_os_error());
                    }
                    unsafe { t.assume_init() }
                }};
            }
            let mut term = tcgetattr!();
            let term_orig = tcgetattr!();
            term.c_lflag &= !libc::ECHO;
            term.c_lflag |= libc::ECHONL;
            if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &term) } != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(Self {
                fd,
                termios: term_orig,
            })
        }
    }

    impl Drop for Hidden {
        fn drop(&mut self) {
            unsafe { libc::tcsetattr(self.fd, libc::TCSANOW, &self.termios) };
        }
    }

    pub fn read_password() -> io::Result<String> {
        let tty = File::open("/dev/tty")?;
        let fd = tty.as_raw_fd();
        let mut reader = io::BufReader::new(tty);
        let mut password = super::EraseableString::new();
        let hidden = Hidden::new(fd)?;
        reader.read_line(&mut password)?;
        drop(hidden);
        super::fix_line(password.into_inner())
    }

    pub fn print_tty(prompt: &str) -> io::Result<()> {
        let mut stream = OpenOptions::new().write(true).open("/dev/tty")?;
        stream.write_all(prompt.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}

#[cfg(windows)]
mod inner {
    use std::{
        fs::File,
        io::{self, BufRead, BufReader, Write},
        os::windows::io::FromRawHandle,
    };
    use windows_sys::Win32::{
        Foundation::{GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE},
        Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
        System::Console::{
            GetConsoleMode, SetConsoleMode, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT,
        },
    };

    // CONIN$ as UTF-16
    const CONIN: &[u16; 7] = &[67, 79, 78, 73, 78, 36, 0];
    // CONOUT$ as UTF-16
    const CONOUT: &[u16; 8] = &[67, 79, 78, 79, 85, 84, 36, 0];

    struct Hidden {
        mode: u32,
        handle: HANDLE,
    }

    impl Hidden {
        fn new(handle: HANDLE) -> io::Result<Self> {
            let mut mode = 0;
            if unsafe { GetConsoleMode(handle, &mut mode) } == 0 {
                return Err(io::Error::last_os_error());
            }
            let new_mode = ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT;
            if unsafe { SetConsoleMode(handle, new_mode) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(Self { mode, handle })
        }
    }

    impl Drop for Hidden {
        fn drop(&mut self) {
            unsafe { SetConsoleMode(self.handle, self.mode) };
        }
    }

    pub fn read_password() -> io::Result<String> {
        let handle = unsafe {
            CreateFileW(
                CONIN.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                core::ptr::null(),
                OPEN_EXISTING,
                0,
                INVALID_HANDLE_VALUE,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let mut stream = BufReader::new(unsafe { File::from_raw_handle(handle as _) });
        let mut password = super::EraseableString::new();
        let hidden = Hidden::new(handle)?;
        let ret = stream.read_line(&mut password);
        println!();
        if let Err(e) = ret {
            return Err(e);
        }
        core::mem::drop(hidden);
        super::fix_line(password.into_inner())
    }

    pub fn print_tty(prompt: &str) -> io::Result<()> {
        let handle = unsafe {
            CreateFileW(
                CONOUT.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                core::ptr::null(),
                OPEN_EXISTING,
                0,
                INVALID_HANDLE_VALUE,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let mut stream = unsafe { File::from_raw_handle(handle as _) };
        stream.write_all(prompt.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}

pub fn get() -> std::io::Result<String> {
    if crate::util::is_a_tty(false) {
        inner::print_tty("Password: ")?;
        inner::read_password()
    } else {
        let mut password = EraseableString::new();
        std::io::stdin().read_line(&mut password)?;
        fix_line(password.into_inner())
    }
}
