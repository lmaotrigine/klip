use crate::error::Error;
use blake2::digest::{Mac, typenum::U8};
use core::{net::SocketAddr, num::NonZeroUsize, time::Duration};
use ed25519_dalek::{SigningKey, VerifyingKey};

/// A builder for a [`Client`][crate::Client].
#[must_use]
#[derive(Clone, Copy)]
pub struct Client {
    addr: SocketAddr,
    encrypt_sk: [u8; 32],
    psk: [u8; 32],
    sign_pk: [u8; 32],
    sign_sk: [u8; 32],
    timeout: Duration,
    data_timeout: Duration,
    ttl: Duration,
    max_stream_bytes: u64,
    max_stream_duration: Duration,
}

impl Client {
    /// Create a new builder. The order for the keys is important and not
    /// intuitive because they all have the same type. Consider using the
    /// [`client!`][crate::client!] macro to pass parameters as `key: value`
    /// pairs.
    pub const fn new(
        psk: [u8; 32],
        encrypt_sk: [u8; 32],
        sign_pk: [u8; 32],
        sign_sk_: [u8; 32],
    ) -> Self {
        Self {
            addr: crate::DEFAULT_CONNECT,
            encrypt_sk,
            psk,
            sign_pk,
            sign_sk: sign_sk_,
            timeout: Duration::from_secs(10),
            data_timeout: Duration::from_hours(1),
            ttl: crate::DEFAULT_TTL,
            max_stream_bytes: crate::DEFAULT_MAX_STREAM_BYTES,
            max_stream_duration: crate::DEFAULT_MAX_STREAM_DURATION,
        }
    }

    /// Specify he address of the server to connect to. The default is
    /// `127.0.0.1:8075`.
    pub const fn addr(mut self, addr: SocketAddr) -> Self {
        self.addr = addr;
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

    fn make_eskid(&self) -> u64 {
        let mut hasher = blake2::Blake2bMac::<U8>::new_with_salt_and_personal(
            None,
            &[],
            crate::DOMAIN.as_bytes(),
        )
        .expect("invalid params in blake2b");
        hasher.update(&self.encrypt_sk);
        u64::from_le_bytes(hasher.finalize().into_bytes().into())
    }

    /// Build a [`Client`] from the builder.
    ///
    /// # Errors
    ///
    /// If the public key bytes were not valid, [`Error::InvalidField`] is
    /// returned.
    pub fn build(self) -> Result<crate::Client, Error> {
        Ok(crate::Client {
            addr: self.addr,
            encrypt_sk: self.encrypt_sk,
            encrypt_sk_id: self.make_eskid(),
            psk: self.psk,
            sign_pk: VerifyingKey::from_bytes(&self.sign_pk)
                .map_err(|_| Error::InvalidField("sign_pk"))?,
            sign_sk: SigningKey::from_bytes(&self.sign_sk),
            timeout: self.timeout,
            data_timeout: self.data_timeout,
            ttl: self.ttl,
            max_stream_bytes: self.max_stream_bytes,
            max_stream_duration: self.max_stream_duration,
        })
    }
}

/// A builder for a [`Server`][crate::Server].
#[must_use]
#[derive(Clone, Copy)]
pub struct Server {
    addr: SocketAddr,
    max_len_mb: u64,
    max_clients: Option<NonZeroUsize>,
    psk: [u8; 32],
    sign_pk: [u8; 32],
    timeout: Duration,
    data_timeout: Duration,
    ttl: Duration,
    max_stream_bytes: u64,
    max_stream_duration: Duration,
    max_waiting_pullers: usize,
}

impl Server {
    /// Create a new builder. The order for the keys is important and not
    /// intuitive because they all have the same type. Consider using the
    /// [`server!`][crate::server] macro to pass parameters as `key: value`
    /// pairs.
    pub const fn new(psk: [u8; 32], sign_pk: [u8; 32]) -> Self {
        Self {
            addr: crate::DEFAULT_LISTEN,
            max_len_mb: 0,
            max_clients: None,
            psk,
            sign_pk,
            timeout: Duration::from_secs(10),
            data_timeout: Duration::from_hours(1),
            ttl: crate::DEFAULT_TTL,
            max_stream_bytes: crate::DEFAULT_MAX_STREAM_BYTES,
            max_stream_duration: crate::DEFAULT_MAX_STREAM_DURATION,
            max_waiting_pullers: crate::DEFAULT_MAX_WAITING_PULLERS,
        }
    }

    /// The address to listen on. The default is `0.0.0.0:8075`.
    pub const fn addr(mut self, addr: SocketAddr) -> Self {
        self.addr = addr;
        self
    }

    /// The maximum content length that the server should accept, in
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

impl core::fmt::Debug for Client {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let display_secrets = f.alternate();
        let mut s = f.debug_struct("ClientBuilder");
        s.field("addr", &self.addr)
            .field("timeout", &self.timeout)
            .field("data_timeout", &self.data_timeout)
            .field("ttl", &self.ttl)
            .field("max_stream_bytes", &self.max_stream_bytes)
            .field("max_stream_duration", &self.max_stream_duration);
        if display_secrets {
            s.field("encrypt_sk", &self.encrypt_sk)
                .field("psk", &self.psk)
                .field("sign_pk", &self.sign_pk)
                .field("sign_sk", &self.sign_sk);
        }
        s.finish()
    }
}

impl core::fmt::Debug for Server {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let display_secrets = f.alternate();
        let mut s = f.debug_struct("ConfigBuilder");
        s.field("addr", &self.addr)
            .field("max_clients", &self.max_clients)
            .field("max_len_mb", &self.max_len_mb)
            .field("timeout", &self.timeout)
            .field("data_timeout", &self.data_timeout)
            .field("ttl", &self.ttl)
            .field("max_stream_bytes", &self.max_stream_bytes)
            .field("max_stream_duration", &self.max_stream_duration)
            .field("max_waiting_pullers", &self.max_waiting_pullers);
        if display_secrets {
            s.field("psk", &self.psk).field("sign_pk", &self.sign_pk);
        }
        s.finish()
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __expand {
    (@client [psk: [$psk:expr]] [encrypt_sk: [$encrypt_sk:expr]] [sign_pk: [$sign_pk:expr]] [sign_sk: [$sign_sk:expr]]; $($key:ident : $val:expr),*$(,)?) => {
        $crate::ClientBuilder::new($psk, $encrypt_sk, $sign_pk, $sign_sk)
        $(
            .$key($val)
        )*
        .build()
    };
    (@client [psk: []] $encrypt_sk:tt $sign_pk:tt $sign_sk:tt;psk : $psk:expr $(, $($rest:tt)*)?) => {
        $crate::__expand!(@client [psk: [$psk]] $encrypt_sk $sign_pk $sign_sk; $($($rest)*)?)
    };
    (@client $psk:tt [encrypt_sk: []] $sign_pk:tt $sign_sk:tt;encrypt_sk : $encrypt_sk:expr $(, $($rest:tt)*)?) => {
        $crate::__expand!(@client $psk [encrypt_sk: [$encrypt_sk]] $sign_pk $sign_sk; $($($rest)*)?)
    };
    (@client $psk:tt $encrypt_sk:tt [sign_pk: []] $sign_sk:tt;sign_pk : $sign_pk:expr $(, $($rest:tt)*)?) => {
        $crate::__expand!(@client $psk $encrypt_sk [sign_pk: [$sign_pk]] $sign_sk; $($($rest)*)?)
    };
    (@client $psk:tt $encrypt_sk:tt $sign_pk:tt [sign_sk: []];sign_sk : $sign_sk:expr $(, $($rest:tt)*)?) => {
        $crate::__expand!(@client $psk $encrypt_sk $sign_pk [sign_sk: [$sign_sk]]; $($($rest)*)?)
    };

    (@server [psk: [$psk:expr]] [sign_pk: [$sign_pk:expr]]; $($key:ident : $val:expr),*$(,)?) => {
        $crate::ServerBuilder::new($psk, $sign_pk)
        $(
            .$key($val)
        )*
        .build()
    };
    (@server [psk: []] $sign_pk:tt;psk : $psk:expr $(, $($rest:tt)*)?) => {
        $crate::__expand!(@server [psk: [$psk]] $sign_pk; $($($rest)*)?)
    };
    (@server $psk:tt [sign_pk: []];sign_pk : $sign_pk:expr $(, $($rest:tt)*)?) => {
        $crate::__expand!(@server $psk [sign_pk: [$sign_pk]]; $($($rest)*)?)
    };
}

/// Create a new [`Client`][crate::Client] by specifying the required keys.
///
/// This is a convenience macro over [`ClientBuilder`][crate::ClientBuilder] to
/// make it easier to specify the required keys without having to remember the
/// argument order of [`new()`][crate::ClientBuilder::new].
///
/// Due to limitations of the macro system, the required arguments `psk`,
/// `encrypt_sk`, `sign_pk`, and `sign_sk` must be specified **before** any
/// other arguments.
///
/// # Example
///
/// ```rust
/// # use klip::client;
/// // these are obviously invalid keys, use your own!
/// let client = client!(
///     psk: [0; 32],
///     sign_pk: [1; 32],
///     encrypt_sk: [2; 32],
///     sign_sk: [3; 32],
///     max_stream_bytes: 2 << 20
/// ).unwrap();
/// ```
#[macro_export]
macro_rules! client {
    ($($args:tt)+) => {
        $crate::__expand!(@client [psk: []] [encrypt_sk: []] [sign_pk: []] [sign_sk: []]; $($args)+)
    };
}

/// Create a new [`Server`][crate::Server] by specifying the required keys.
///
/// This is a convenience macro over [`ServerBuilder`][crate::ServerBuilder] to
/// make it easier to specify the required keys without having to remember the
/// argument order of [`new()`][crate::ServerBuilder::new].
///
/// Due to limitations of the macro system, the required arguments `psk` and
/// `sign_pk` must be specified **before** any other arguments.
///
/// # Example
///
/// ```rust
/// # use klip::server;
/// // these are obviously invalid keys, use your own!
/// let server = server!(psk: [0; 32], sign_pk: [1; 32], max_len_mb: 100).unwrap();
/// ```
#[macro_export]
macro_rules! server {
    ($($args:tt)+) => {
        $crate::__expand!(@server [psk: []] [sign_pk: []]; $($args)+)
    };
}
