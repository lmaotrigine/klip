use std::time::Duration;
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
