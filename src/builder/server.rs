use super::{Debug, State};
use crate::Error;
use core::{net::SocketAddr, num::NonZeroUsize, time::Duration};
use ed25519_dalek::VerifyingKey;

/// A builder for a [`Server`][crate::Server].
#[must_use]
#[derive(Clone, Copy)]
pub struct Server<Psk: State = (), SignPk: State = ()> {
    addr: SocketAddr,
    max_len_mb: u64,
    max_clients: Option<NonZeroUsize>,
    psk: Psk,
    sign_pk: SignPk,
    timeout: Duration,
    data_timeout: Duration,
    ttl: Duration,
    max_stream_bytes: u64,
    max_stream_duration: Duration,
    max_waiting_pullers: usize,
}

impl Server {
    /// Create a new builder.
    pub const fn new() -> Self {
        Self {
            addr: crate::DEFAULT_LISTEN,
            max_len_mb: 0,
            max_clients: None,
            psk: (),
            sign_pk: (),
            timeout: Duration::from_secs(10),
            data_timeout: Duration::from_hours(1),
            ttl: crate::DEFAULT_TTL,
            max_stream_bytes: crate::DEFAULT_MAX_STREAM_BYTES,
            max_stream_duration: crate::DEFAULT_MAX_STREAM_DURATION,
            max_waiting_pullers: crate::DEFAULT_MAX_WAITING_PULLERS,
        }
    }
}

impl<Psk: State, SignPk: State> Server<Psk, SignPk> {
    /// Specify the 256-bit pre-shared key. This is required.
    pub const fn psk(self, psk: [u8; 32]) -> Server<[u8; 32], SignPk> {
        Server {
            addr: self.addr,
            max_len_mb: self.max_len_mb,
            max_clients: self.max_clients,
            psk,
            sign_pk: self.sign_pk,
            timeout: self.timeout,
            data_timeout: self.data_timeout,
            ttl: self.ttl,
            max_stream_bytes: self.max_stream_bytes,
            max_stream_duration: self.max_stream_duration,
            max_waiting_pullers: self.max_waiting_pullers,
        }
    }

    /// Specify the Ed25519 public key as a 32-byte array. This is required.
    pub const fn sign_pk(self, sign_pk: [u8; 32]) -> Server<Psk, [u8; 32]> {
        Server {
            addr: self.addr,
            max_len_mb: self.max_len_mb,
            max_clients: self.max_clients,
            psk: self.psk,
            sign_pk,
            timeout: self.timeout,
            data_timeout: self.data_timeout,
            ttl: self.ttl,
            max_stream_bytes: self.max_stream_bytes,
            max_stream_duration: self.max_stream_duration,
            max_waiting_pullers: self.max_waiting_pullers,
        }
    }

    /// Specify the address to listen on. The default is `0.0.0.0:8075`.
    pub const fn addr(mut self, addr: SocketAddr) -> Self {
        self.addr = addr;
        self
    }

    /// Specify the maximum content length that the server should accept, in
    /// mebibytes. A value of `0` disables this limit.
    ///
    /// The default is no limit.
    pub const fn max_len_mb(mut self, max_len: u64) -> Self {
        self.max_len_mb = max_len;
        self
    }

    /// Specify the maximum number of clients that the server should handle
    /// simultaneously. The default is 10.
    pub const fn max_clients(mut self, max_clients: NonZeroUsize) -> Self {
        self.max_clients = Some(max_clients);
        self
    }

    /// Specify the connection timeout. The default is 10 seconds.
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Specify the data transmission timeout. This applies to individual chunks
    /// when used in streaming operations. The default is 1 hour.
    pub const fn data_timeout(mut self, timeout: Duration) -> Self {
        self.data_timeout = timeout;
        self
    }

    /// Specify the time-to-live for content. This is the maximum amount of time
    /// before content is considered expired and is no longer available for
    /// retrieval. The default is 7 days.
    pub const fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Specify the maximum number of bytes that can be streamed in a single
    /// operation. The default is 10GiB.
    pub const fn max_stream_bytes(mut self, max_stream_bytes: u64) -> Self {
        self.max_stream_bytes = max_stream_bytes;
        self
    }

    /// Specify the maximum duration for a streaming operation. The default is
    /// 24 hours.
    pub const fn max_stream_duration(mut self, duration: Duration) -> Self {
        self.max_stream_duration = duration;
        self
    }

    /// Specify the maximum number of pullers that can wait for a single stream.
    /// The default is 100.
    pub const fn max_waiting_pullers(mut self, max_waiting_pullers: usize) -> Self {
        self.max_waiting_pullers = max_waiting_pullers;
        self
    }
}

impl Server<[u8; 32], [u8; 32]> {
    /// Build a [`Server`][crate::Server] from the builder.
    ///
    /// # Errors
    ///
    /// If the public key bytes were not valid, [`Error::InvalidField`] is
    /// returned.
    pub fn build(self) -> Result<crate::Server, Error> {
        Ok(crate::Server {
            addr: self.addr,
            max_len: self.max_len_mb << 20,
            max_clients: self.max_clients.map_or(10, core::num::NonZero::get),
            psk: self.psk,
            sign_pk: VerifyingKey::from_bytes(&self.sign_pk)
                .map_err(|_| Error::InvalidField("sign_pk"))?,
            timeout: self.timeout,
            data_timeout: self.timeout,
            ttl: self.ttl,
            max_stream_bytes: self.max_stream_bytes,
            max_stream_duration: self.max_stream_duration,
            max_waiting_pullers: self.max_waiting_pullers,
        })
    }
}

impl<Psk: State, SignPk: State> core::fmt::Debug for Server<Psk, SignPk> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ServerBuilder")
            .field("addr", &self.addr)
            .field("max_len_mb", &self.max_len_mb)
            .field("max_clients", &self.max_clients)
            .field("psk", &Debug(self.psk))
            .field("sign_pk", &Debug(self.sign_pk))
            .field("timeout", &self.timeout)
            .field("data_timeout", &self.data_timeout)
            .field("ttl", &self.ttl)
            .field("max_stream_bytes", &self.max_stream_bytes)
            .field("max_stream_duration", &self.max_stream_duration)
            .field("max_waiting_pullers", &self.max_waiting_pullers)
            .finish()
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
