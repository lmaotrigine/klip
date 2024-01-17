use std::{
    future::Future,
    io,
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    time::{sleep_until, Instant, Sleep},
};

#[derive(Debug)]
struct TimeoutState {
    timeout: Option<Duration>,
    cur: Sleep,
    active: bool,
}

struct TimeoutStateProjection<'a>
where
    TimeoutState: 'a,
{
    timeout: &'a mut Option<Duration>,
    cur: Pin<&'a mut Sleep>,
    active: &'a mut bool,
}

impl TimeoutState {
    #[inline]
    fn new() -> Self {
        Self {
            timeout: None,
            cur: sleep_until(Instant::now()),
            active: false,
        }
    }

    #[inline]
    fn project<'a>(self: Pin<&'a mut Self>) -> TimeoutStateProjection<'a> {
        unsafe {
            let Self {
                timeout,
                cur,
                active,
            } = self.get_unchecked_mut();
            TimeoutStateProjection {
                timeout,
                cur: Pin::new_unchecked(cur),
                active,
            }
        }
    }

    #[inline]
    fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = Some(timeout);
    }

    #[inline]
    fn reset(self: Pin<&mut Self>) {
        let this = self.project();
        if *this.active {
            *this.active = false;
            this.cur.reset(Instant::now());
        }
    }

    #[inline]
    fn poll_check(self: Pin<&mut Self>, cx: &mut Context<'_>) -> io::Result<()> {
        let mut this = self.project();
        let timeout = match this.timeout {
            Some(timeout) => *timeout,
            None => return Ok(()),
        };
        if !*this.active {
            this.cur.as_mut().reset(Instant::now() + timeout);
            *this.active = true;
        }
        match this.cur.poll(cx) {
            Poll::Ready(()) => Err(io::Error::from(io::ErrorKind::TimedOut)),
            Poll::Pending => Ok(()),
        }
    }
}

#[derive(Debug)]
pub struct TimeoutIO<IO> {
    inner: IO,
    state: TimeoutState,
}

struct TimeoutIOProjection<'a, IO>
where
    TimeoutIO<IO>: 'a,
{
    inner: Pin<&'a mut IO>,
    state: Pin<&'a mut TimeoutState>,
}

impl<IO> TimeoutIO<IO> {
    pub fn new(buf_io: IO) -> Self {
        Self {
            inner: buf_io,
            state: TimeoutState::new(),
        }
    }

    #[inline]
    fn project<'a>(self: Pin<&'a mut Self>) -> TimeoutIOProjection<'a, IO> {
        unsafe {
            let Self { inner, state } = self.get_unchecked_mut();
            TimeoutIOProjection {
                inner: Pin::new_unchecked(inner),
                state: Pin::new_unchecked(state),
            }
        }
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.state.set_timeout(timeout);
    }

    pub fn get_mut(&mut self) -> &mut IO {
        &mut self.inner
    }
}

impl<IO: AsyncRead> AsyncRead for TimeoutIO<IO> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.project();
        let r = this.inner.poll_read(cx, buf);
        match r {
            Poll::Pending => this.state.poll_check(cx)?,
            Poll::Ready(_) => this.state.reset(),
        }
        r
    }
}

impl<IO: AsyncWrite> AsyncWrite for TimeoutIO<IO> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        self.project().inner.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        self.project().inner.poll_shutdown(cx)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<Result<usize, io::Error>> {
        self.project().inner.poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }
}

// various utils to check that our pin projections are safe
mod __sanity_checks__ {
    use super::{Duration, Sleep, TimeoutIO, TimeoutState};
    use core::marker::PhantomData;

    struct AssertUnpin<T: ?Sized>(PhantomData<T>);
    impl<T: ?Sized> Unpin for AssertUnpin<T> {}
    trait NoAutoDropImpl {}
    #[allow(drop_bounds)]
    impl<T: Drop> NoAutoDropImpl for T {}

    struct _TimeoutStateSafetyCheck<'a> {
        _dummy: PhantomData<&'a ()>,
        _timeout: AssertUnpin<Option<Duration>>,
        _cur: Sleep,
        _active: AssertUnpin<bool>,
    }
    impl<'a> Unpin for TimeoutState where _TimeoutStateSafetyCheck<'a>: Unpin {}
    impl NoAutoDropImpl for TimeoutState {}
    struct _TimeoutIOSafetyCheck<'a, IO> {
        _dummy: PhantomData<&'a ()>,
        _inner: IO,
        _state: TimeoutState,
    }
    impl<'a, IO> Unpin for TimeoutIO<IO> where _TimeoutIOSafetyCheck<'a, IO>: Unpin {}
    impl<IO> NoAutoDropImpl for TimeoutIO<IO> {}
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
