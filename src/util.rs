use core::{future::Future, time::Duration};
use tokio::{
    io::BufStream,
    net::TcpStream,
    time::{Instant, timeout_at},
};

/// Read bytes from a source.
///
/// This is analogous to [`AsyncReadExt`][tokio::io::AsyncReadExt], but does not
/// require [`AsyncRead`][tokio::io::AsyncRead] to be implemented, and only
/// contains the methods that are actually used by klip.
pub trait Reader: Unpin {
    /// Pulls some bytes from this source into the specified buffer, returning
    /// how many bytes were read.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    /// ```
    ///
    /// Implementors should provide the same guarantees and caveats as
    /// [`AsyncReadExt::read()`][tokio::io::AsyncReadExt::read]. Consult the
    /// documentation for that method for more information.
    fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = std::io::Result<usize>>;
    /// Reads the exact number of bytes required to fill `buf`.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
    /// ```
    ///
    /// Implementors should provide the same guarantees and caveats as
    /// [`AsyncReadExt::read_exact()`][tokio::io::AsyncReadExt::read_exact].
    /// Consult the documentation for that method for more information.
    fn read_exact(&mut self, buf: &mut [u8]) -> impl Future<Output = std::io::Result<usize>>;
    /// Reads all bytes until EOF in this source, placing them into `buf`.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize>;
    /// ```
    ///
    /// Implementors should provide the same guarantees and caveats as
    /// [`AsyncReadExt::read_to_end()`][tokio::io::AsyncReadExt::read_to_end()].
    /// Consult the documentation for that method for more information.
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> impl Future<Output = std::io::Result<usize>>;
}

/// Write bytes to a sink.
///
/// This is analogous to [`tokio::io::AsyncWriteExt`], but does not require
/// [`tokio::io::AsyncWrite`] to be implemented, and only contains the methods
/// that are actually used by klip.
pub trait Writer: Unpin {
    /// Attempts to write an entire buffer into this writer.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()>;
    /// ```
    ///
    /// Implementors should provide the same guarantees and caveats as
    /// [`AsyncWriteExt::write_all()`][tokio::io::AsyncWriteExt::write_all()].
    /// Consult the documentation for that method for more information.
    fn write_all(&mut self, buf: &[u8]) -> impl Future<Output = std::io::Result<()>>;
    /// Flushes this output stream, ensuring that all intermediately buffered
    /// contents reach their destination.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn flush(&mut self) -> std::io::Result<()>;
    /// ```
    ///
    /// Implementors should provide the same guarantees and caveats as
    /// [`AsyncWriteExt::flush()`][tokio::io::AsyncWriteExt::flush()]. Consult
    /// the documentation for that method for more information.
    fn flush(&mut self) -> impl Future<Output = std::io::Result<()>>;
}

impl<R: tokio::io::AsyncRead + Unpin> Reader for R {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        tokio::io::AsyncReadExt::read(self, buf).await
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        tokio::io::AsyncReadExt::read_exact(self, buf).await
    }

    async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        tokio::io::AsyncReadExt::read_to_end(self, buf).await
    }
}

impl<W: tokio::io::AsyncWrite + Unpin> Writer for W {
    async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        tokio::io::AsyncWriteExt::write_all(self, buf).await
    }

    async fn flush(&mut self) -> std::io::Result<()> {
        tokio::io::AsyncWriteExt::flush(self).await
    }
}

pub struct Stream {
    inner: BufStream<TcpStream>,
    timeout: Option<Instant>,
}

impl Stream {
    pub fn new(stream: TcpStream) -> Self {
        Self { inner: BufStream::new(stream), timeout: None }
    }

    pub fn set_timeout(&mut self, dur: Duration) {
        self.timeout = Some(Instant::now() + dur);
    }

    pub const fn clear_timeout(&mut self) {
        self.timeout = None;
    }

    pub async fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(timeout) = self.timeout {
            timeout_at(timeout, self.inner.read_exact(buf)).await?
        } else {
            self.inner.read_exact(buf).await
        }
    }

    pub async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        if let Some(timeout) = self.timeout {
            timeout_at(timeout, self.inner.write_all(buf)).await?
        } else {
            self.inner.write_all(buf).await
        }
    }

    pub async fn flush(&mut self) -> std::io::Result<()> {
        if let Some(timeout) = self.timeout {
            timeout_at(timeout, self.inner.flush()).await?
        } else {
            self.inner.flush().await
        }
    }

    pub fn peer_addr(&self) -> std::io::Result<core::net::SocketAddr> {
        self.inner.get_ref().peer_addr()
    }

    pub async fn shutdown(mut self) -> std::io::Result<()> {
        tokio::io::AsyncWriteExt::shutdown(&mut self.inner).await
    }
}

struct Hex<'a> {
    inner: core::slice::Iter<'a, u8>,
    next: Option<u8>,
}

impl Hex<'_> {
    const TABLE: &'static [u8; 16] = b"0123456789abcdef";
}

impl Iterator for Hex<'_> {
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

impl core::iter::ExactSizeIterator for Hex<'_> {
    fn len(&self) -> usize {
        let mut length = self.inner.len() * 2;
        if self.next.is_some() {
            length += 1;
        }
        length
    }
}

pub fn hex<'a>(inp: &[u8], out: &'a mut [u8]) -> &'a str {
    let iter = Hex { inner: inp.iter(), next: None };
    for (i, j) in iter.zip(out.iter_mut()) {
        *j = i;
    }
    // SAFETY: Hex iterator only produces ASCII characters
    #[expect(unsafe_code)]
    unsafe {
        core::str::from_utf8_unchecked(out)
    }
}

#[cfg(feature = "cli")]
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
    if !bytes.len().is_multiple_of(2) {
        return Err(());
    }
    if bytes.len() != buf.len() * 2 {
        return Err(());
    }
    for (i, b) in buf.iter_mut().enumerate() {
        *b = (decode_char(bytes[i * 2])? << 4) | decode_char(bytes[i * 2 + 1])?;
    }
    Ok(())
}

#[cfg(feature = "cli")]
pub fn prompt(prompt: &str) -> Result<String, std::io::Error> {
    use std::io::Write;
    let mut ret = String::new();
    print!("{prompt}");
    let _ = std::io::stdout().flush();
    std::io::stdin().read_line(&mut ret)?;
    while ret.ends_with(['\n', '\r']) {
        ret.pop();
    }
    Ok(ret)
}
