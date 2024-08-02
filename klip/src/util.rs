use std::{path::PathBuf, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufStream},
    net::TcpStream,
    time::{timeout_at, Instant},
};

pub struct Stream {
    inner: BufStream<TcpStream>,
    timeout: Option<Instant>,
}

macro_rules! timed_out {
    () => {
        std::io::Error::from(std::io::ErrorKind::TimedOut)
    };
}

impl Stream {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            inner: BufStream::new(stream),
            timeout: None,
        }
    }

    pub fn set_timeout(&mut self, dur: Duration) {
        self.timeout = Some(Instant::now() + dur);
    }

    pub async fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(timeout) = self.timeout {
            timeout_at(timeout, self.inner.read_exact(buf))
                .await
                .map_err(|_| timed_out!())?
        } else {
            self.inner.read_exact(buf).await
        }
    }

    pub async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        if let Some(timeout) = self.timeout {
            timeout_at(timeout, self.inner.read_to_end(buf))
                .await
                .map_err(|_| timed_out!())?
        } else {
            self.inner.read_to_end(buf).await
        }
    }

    pub async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        if let Some(timeout) = self.timeout {
            timeout_at(timeout, self.inner.write_all(buf))
                .await
                .map_err(|_| timed_out!())?
        } else {
            self.inner.write_all(buf).await
        }
    }

    pub async fn flush(&mut self) -> std::io::Result<()> {
        if let Some(timeout) = self.timeout {
            timeout_at(timeout, self.inner.flush())
                .await
                .map_err(|_| timed_out!())?
        } else {
            self.inner.flush().await
        }
    }

    pub fn peer_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.inner.get_ref().peer_addr()
    }

    pub async fn shutdown(mut self) -> std::io::Result<()> {
        self.inner.shutdown().await
    }
}

#[cfg(unix)]
#[inline]
pub fn is_a_tty(stderr: bool) -> bool {
    unsafe {
        libc::isatty(if stderr {
            libc::STDERR_FILENO
        } else {
            libc::STDOUT_FILENO
        }) != 0
    }
}

#[cfg(windows)]
pub fn is_a_tty(stderr: bool) -> bool {
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

#[cfg(any(unix, target_os = "redox"))]
pub fn home_dir() -> Option<PathBuf> {
    #[allow(deprecated)]
    std::env::home_dir()
}

#[cfg(all(windows, not(target_vendor = "uwp")))]
fn home_dir_crt() -> Option<PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::{
        Foundation::{MAX_PATH, S_OK},
        UI::Shell::{SHGetFolderPathW, CSIDL_PROFILE},
    };
    extern "C" {
        fn wcslen(buf: *const u16) -> usize;
    }
    let mut path = Vec::with_capacity(MAX_PATH as usize);
    #[allow(clippy::cast_possible_wrap)]
    unsafe {
        match SHGetFolderPathW(0, CSIDL_PROFILE as i32, 0, 0, path.as_mut_ptr()) {
            S_OK => {
                let len = wcslen(path.as_ptr());
                path.set_len(len);
                let s = std::ffi::OsString::from_wide(&path);
                Some(PathBuf::from(s))
            }
            _ => None,
        }
    }
}

#[cfg(all(windows, target_vendor = "uwp"))]
#[allow(clippy::missing_const_for_fn)]
fn home_dir_crt() -> Option<PathBuf> {
    None
}

#[cfg(windows)]
pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(home_dir_crt)
}

struct Hex<'a> {
    inner: core::slice::Iter<'a, u8>,
    next: Option<u8>,
}

impl<'a> Hex<'a> {
    const TABLE: &'static [u8; 16] = b"0123456789abcdef";
}

impl<'a> Iterator for Hex<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next.take() {
            Some(c) => Some(c),
            None => self.inner.next().map(|b| {
                let current = Self::TABLE[(b >> 4) as usize];
                self.next = Some(Self::TABLE[(b & 0xf) as usize]);
                current
            }),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.len();
        (length, Some(length))
    }
}

impl<'a> core::iter::ExactSizeIterator for Hex<'a> {
    fn len(&self) -> usize {
        let mut length = self.inner.len() * 2;
        if self.next.is_some() {
            length += 1;
        }
        length
    }
}

pub fn hex(inp: &[u8], out: &mut [u8]) {
    let iter = Hex {
        inner: inp.iter(),
        next: None,
    };
    for (i, j) in iter.zip(out.iter_mut()) {
        *j = i;
    }
}

pub fn from_hex(s: &str, buf: &mut [u8]) -> Result<(), ()> {
    const fn decode_char(b: u8) -> Result<u8, ()> {
        match b {
            b'a'..=b'f' => Ok(b - b'a' + 10),
            b'A'..=b'F' => Ok(b - b'A' + 10),
            b'0'..=b'9' => Ok(b - b'0'),
            _ => Err(()),
        }
    }
    let bytes = s.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err(());
    }
    if bytes.len() != buf.len() * 2 {
        return Err(());
    }
    for (i, b) in buf.iter_mut().enumerate() {
        *b = decode_char(bytes[i * 2])? << 4 | decode_char(bytes[i * 2 + 1])?;
    }
    Ok(())
}
