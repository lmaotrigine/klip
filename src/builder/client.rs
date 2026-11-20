use super::{Debug, State};
use crate::Error;
use blake2::digest::{Mac, typenum::U8};
use core::{net::SocketAddr, time::Duration};
use ed25519_dalek::{SigningKey, VerifyingKey};

// 4 generics with bounds looks weird in rustdoc when inlined, so use where
// clauses here.

/// A builder for a [`Client`][crate::Client].
#[must_use]
#[derive(Clone, Copy)]
pub struct Client<Psk = (), EncryptSk = (), SignPk = (), SignSk = ()>
where
    Psk: State,
    EncryptSk: State,
    SignPk: State,
    SignSk: State,
{
    addr: SocketAddr,
    encrypt_sk: EncryptSk,
    psk: Psk,
    sign_pk: SignPk,
    sign_sk: SignSk,
    timeout: Duration,
    data_timeout: Duration,
    ttl: Duration,
    max_stream_bytes: u64,
    max_stream_duration: Duration,
}

impl Client {
    /// Create a new builder.
    pub const fn new() -> Self {
        Self {
            addr: crate::DEFAULT_CONNECT,
            encrypt_sk: (),
            psk: (),
            sign_pk: (),
            sign_sk: (),
            timeout: Duration::from_secs(10),
            data_timeout: Duration::from_hours(1),
            ttl: crate::DEFAULT_TTL,
            max_stream_bytes: crate::DEFAULT_MAX_STREAM_BYTES,
            max_stream_duration: crate::DEFAULT_MAX_STREAM_DURATION,
        }
    }
}

impl<Psk, EncryptSk, SignPk, SignSk> Client<Psk, EncryptSk, SignPk, SignSk>
where
    Psk: State,
    EncryptSk: State,
    SignPk: State,
    SignSk: State,
{
    /// Specify the 256-bit pre-shared key. This is required.
    pub const fn psk(self, psk: [u8; 32]) -> Client<[u8; 32], EncryptSk, SignPk, SignSk> {
        Client {
            addr: self.addr,
            encrypt_sk: self.encrypt_sk,
            psk,
            sign_pk: self.sign_pk,
            sign_sk: self.sign_sk,
            timeout: self.timeout,
            data_timeout: self.data_timeout,
            ttl: self.ttl,
            max_stream_bytes: self.max_stream_bytes,
            max_stream_duration: self.max_stream_duration,
        }
    }

    /// Specify the ChaCha20-Poly1305 secret key. This is required.
    pub const fn encrypt_sk(self, encrypt_sk: [u8; 32]) -> Client<Psk, [u8; 32], SignPk, SignSk> {
        Client {
            addr: self.addr,
            encrypt_sk,
            psk: self.psk,
            sign_pk: self.sign_pk,
            sign_sk: self.sign_sk,
            timeout: self.timeout,
            data_timeout: self.data_timeout,
            ttl: self.ttl,
            max_stream_bytes: self.max_stream_bytes,
            max_stream_duration: self.max_stream_duration,
        }
    }

    /// Specify the Ed25519 public key as a 32-byte array. This is required.
    pub const fn sign_pk(self, sign_pk: [u8; 32]) -> Client<Psk, EncryptSk, [u8; 32], SignSk> {
        Client {
            addr: self.addr,
            encrypt_sk: self.encrypt_sk,
            psk: self.psk,
            sign_pk,
            sign_sk: self.sign_sk,
            timeout: self.timeout,
            data_timeout: self.data_timeout,
            ttl: self.ttl,
            max_stream_bytes: self.max_stream_bytes,
            max_stream_duration: self.max_stream_duration,
        }
    }

    /// Specify the Ed25519 secret key as a 32-byte array. This is required.
    pub const fn sign_sk(self, sign_sk: [u8; 32]) -> Client<Psk, EncryptSk, SignPk, [u8; 32]> {
        Client {
            addr: self.addr,
            encrypt_sk: self.encrypt_sk,
            psk: self.psk,
            sign_pk: self.sign_pk,
            sign_sk,
            timeout: self.timeout,
            data_timeout: self.data_timeout,
            ttl: self.ttl,
            max_stream_bytes: self.max_stream_bytes,
            max_stream_duration: self.max_stream_duration,
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
}

impl Client<[u8; 32], [u8; 32], [u8; 32], [u8; 32]> {
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

impl<Psk, EncryptSk, SignPk, SignSk> core::fmt::Debug for Client<Psk, EncryptSk, SignPk, SignSk>
where
    Psk: State,
    EncryptSk: State,
    SignPk: State,
    SignSk: State,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ClientBuilder")
            .field("addr", &self.addr)
            .field("encrypt_sk", &Debug(self.encrypt_sk))
            .field("psk", &Debug(self.psk))
            .field("sign_pk", &Debug(self.sign_pk))
            .field("sign_sk", &Debug(self.sign_sk))
            .field("timeout", &self.timeout)
            .field("data_timeout", &self.data_timeout)
            .field("ttl", &self.ttl)
            .field("max_stream_bytes", &self.max_stream_bytes)
            .field("max_stream_duration", &self.max_stream_duration)
            .finish()
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
