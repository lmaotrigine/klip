use std::{
    fs::File,
    io::{BufRead, BufReader, Result, Write},
    os::windows::io::FromRawHandle,
};
use windows_sys::Win32::{
    Foundation::{GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
    System::Console::{GetConsoleMode, SetConsoleMode, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT},
};

const CONIN: &[u16; 7] = &[0x0043, 0x004f, 0x004e, 0x0049, 0x004e, 0x0024, 0x0000];
const CONOUT: &[u16; 8] = &[
    0x0043, 0x004f, 0x004e, 0x004f, 0x0055, 0x0054, 0x0024, 0x0000,
];

struct Hidden {
    mode: u32,
    handle: HANDLE,
}

impl Hidden {
    fn new(handle: HANDLE) -> Result<Self> {
        let mut mode = 0;
        if unsafe { GetConsoleMode(handle, &mut mode) } == 0 {
            return Err(std::io::Error::last_os_error());
        }
        let new_mode = ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT;
        if unsafe { SetConsoleMode(handle, new_mode) } == 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self { mode, handle })
    }
}

impl Drop for Hidden {
    fn drop(&mut self) {
        unsafe { SetConsoleMode(self.handle, self.mode) };
    }
}

pub fn read_password() -> Result<String> {
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
        return Err(std::io::Error::last_os_error());
    }
    let mut stream = BufReader::new(unsafe { File::from_raw_handle(handle as _) });
    let mut password = super::Password::new();
    let hidden = Hidden::new(handle)?;
    let ret = stream.read_line(&mut password.0);
    println!();
    ret?;
    core::mem::drop(hidden);
    super::fix_line(password.into_inner())
}

pub fn print_tty(prompt: &str) -> Result<()> {
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
        return Err(std::io::Error::last_os_error());
    }
    let mut stream = unsafe { File::from_raw_handle(handle as _) };
    stream.write_all(prompt.as_bytes())?;
    stream.flush()?;
    Ok(())
}
