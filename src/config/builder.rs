use crate::{client::Client, config::Config, error::Error, server::Server};
use blake2::digest::{Mac, typenum::U8};
use core::{net::SocketAddr, num::NonZeroUsize, time::Duration};
use ed25519_dalek::{SigningKey, VerifyingKey};

macro_rules! some {
    ($self:ident.$field:ident) => {
        $self.$field.ok_or(Error::MissingField(stringify!($field)))
    };
}

/// A builder for either a [`Client`] or a [`Server`].
///
/// This allows you to specify the necessary keys and configuration options.
#[must_use]
#[derive(Clone, Copy)]
pub struct Builder {
    addr: Option<SocketAddr>,
    max_len: Option<u64>,
    max_clients: Option<NonZeroUsize>,
    encrypt_sk: Option<[u8; 32]>,
    encrypt_sk_id: Option<u64>,
    psk: Option<[u8; 32]>,
    sign_pk: Option<[u8; 32]>,
    sign_sk: Option<[u8; 32]>,
    timeout: Duration,
    data_timeout: Duration,
    ttl: Duration,
    max_stream_bytes: u64,
    max_stream_duration: Duration,
    max_waiting_pullers: usize,
}

impl Builder {
    /// Initialise a new [`Builder`] with default values.
    pub const fn new() -> Self {
        Self {
            addr: None,
            max_len: None,
            max_clients: None,
            encrypt_sk: None,
            encrypt_sk_id: None,
            psk: None,
            sign_pk: None,
            sign_sk: None,
            timeout: Duration::from_secs(10),
            data_timeout: Duration::from_hours(1),
            ttl: crate::DEFAULT_TTL,
            max_stream_bytes: crate::DEFAULT_MAX_STREAM_BYTES,
            max_stream_duration: crate::DEFAULT_MAX_STREAM_DURATION,
            max_waiting_pullers: crate::DEFAULT_MAX_WAITING_PULLERS,
        }
    }

    /// Specify the address for the client to connect to, or for the server to
    /// listen on.
    ///
    /// # Applicability
    ///
    /// This parameter applies to both clients and servers.
    ///
    /// This parameter is **optional**. The default values are:
    ///
    /// - `127.0.0.1:8075` for clients
    /// - `0.0.0.0:8075` for servers
    pub const fn addr(mut self, addr: SocketAddr) -> Self {
        self.addr = Some(addr);
        self
    }

    /// Specify the maximum content length that the server should accept, in
    /// mebibytes. A value of `0` disables this limit.
    ///
    /// # Applicability
    ///
    /// This parameter applies only to servers.
    ///
    /// This parameter is **optional**. The default value is `0` (no limit).
    pub const fn max_len_mb(mut self, max_len: u64) -> Self {
        self.max_len = Some(max_len);
        self
    }

    /// Specify the maximum number of clients that the server should handle
    /// simultaneously.
    ///
    /// # Applicability
    ///
    /// This parameter applies only to servers.
    ///
    /// This parameter is **optional**. The default value is `10`.
    pub const fn max_clients(mut self, max_clients: NonZeroUsize) -> Self {
        self.max_clients = Some(max_clients);
        self
    }

    /// Specify the ChaCha20(-Poly1305) secret key.
    ///
    /// # Applicability
    ///
    /// This parameter applies only to clients.
    ///
    /// This parameter is **required**.
    pub const fn encrypt_sk(mut self, encrypt_sk: [u8; 32]) -> Self {
        self.encrypt_sk = Some(encrypt_sk);
        self
    }

    /// Specify a unique identifier for the ChaCha20(-Poly1305) secret key. This
    /// is calculated automatically from the secret key, and need not be
    /// specified unless you are interacting with an older version that did not
    /// support this functionality.
    ///
    /// # Applicability
    ///
    /// This parameter applies only to clients.
    #[deprecated(
        since = "0.4.0",
        note = "encrypt_sk_id is calculated from encrypt_sk and need not be specified manually"
    )]
    pub const fn encrypt_sk_id(mut self, encrypt_sk_id: u64) -> Self {
        self.encrypt_sk_id = Some(encrypt_sk_id);
        self
    }

    /// Specify the 256-bit API key.
    ///
    /// # Applicability
    ///
    /// This parameter applies to both clients and servers.
    ///
    /// This parameter is **required**.
    pub const fn psk(mut self, psk: [u8; 32]) -> Self {
        self.psk = Some(psk);
        self
    }

    /// Specify the Ed25519 public key as a 32-byte array.
    ///
    /// # Applicability
    ///
    /// This parameter applies to both clients and servers.
    ///
    /// This parameter is **required**.
    pub const fn sign_pk(mut self, sign_pk: [u8; 32]) -> Self {
        self.sign_pk = Some(sign_pk);
        self
    }

    /// Specify the Ed25519 secret key as a 32-byte array.
    ///
    /// # Applicability
    ///
    /// This parameter applies only to clients.
    ///
    /// This parameter is **required**.
    pub const fn sign_sk(mut self, sign_sk: [u8; 32]) -> Self {
        self.sign_sk = Some(sign_sk);
        self
    }

    /// Specify the connection timeout.
    ///
    /// # Applicability
    ///
    /// This parameter applies to both clients and servers.
    ///
    /// This parameter is **optional**. The default value is 10 seconds.
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Specify the data transmission timeout. This applies to individual chunks
    /// when used in streaming operations.
    ///
    /// # Applicability
    ///
    /// This parameter applies to both clients and servers.
    ///
    /// This parameter is **optional**. The default value is 1 hour.
    pub const fn data_timeout(mut self, timeout: Duration) -> Self {
        self.data_timeout = timeout;
        self
    }

    /// Specify the time-to-live for content. This is the maximum amount of time
    /// before content is considered expired and is no longer available for
    /// retrieval.
    ///
    /// # Applicability
    ///
    /// This parameter applies to both clients and servers.
    ///
    /// This parameter is **optional**. The default value is 7 days.
    pub const fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Specify the maximum number of bytes that can be streamed in a single
    /// operation.
    ///
    /// # Applicability
    ///
    /// This parameter applies to both clients and servers.
    ///
    /// This parameter is **optional**. The default value is 10GiB.
    pub const fn max_stream_bytes(mut self, max_stream_bytes: u64) -> Self {
        self.max_stream_bytes = max_stream_bytes;
        self
    }

    /// Specify the maximum duration for a streaming operation.
    ///
    /// # Applicability
    ///
    /// This parameter applies to both clients and servers.
    ///
    /// This parameter is **optional**. The default value is 24 hours.
    pub const fn max_stream_duration(mut self, duration: Duration) -> Self {
        self.max_stream_duration = duration;
        self
    }

    /// Specify the maximum number of pullers that can wait for a single stream.
    ///
    /// # Applicability
    ///
    /// This parameter applies only to servers.
    ///
    /// This parameter is **optional**. The default value is 100.
    pub const fn max_waiting_pullers(mut self, max_waiting_pullers: usize) -> Self {
        self.max_waiting_pullers = max_waiting_pullers;
        self
    }

    fn make_eskid(&self) -> Result<u64, Error> {
        if let Some(id) = self.encrypt_sk_id {
            return Ok(id);
        }
        let encrypt_sk = self.encrypt_sk.ok_or(Error::MissingField("encrypt_sk"))?;
        let mut hasher = blake2::Blake2bMac::<U8>::new_with_salt_and_personal(
            None,
            &[],
            crate::DOMAIN.as_bytes(),
        )
        .expect("invalid params in blake2b");
        hasher.update(&encrypt_sk);
        Ok(u64::from_le_bytes(hasher.finalize().into_bytes().into()))
    }

    /// Build a [`Client`] from the builder.
    ///
    /// # Errors
    ///
    /// If any fields required for a client were not provided,
    /// [`Error::MissingField`] is returned.
    pub fn build_client(self) -> Result<Client, Error> {
        let conf = Config {
            connect: self.addr.unwrap_or(crate::DEFAULT_CONNECT),
            listen: crate::DEFAULT_LISTEN,
            max_len: 1,
            max_clients: 1,
            encrypt_sk: some!(self.encrypt_sk)?,
            encrypt_sk_id: self.make_eskid()?,
            psk: some!(self.psk)?,
            sign_pk: VerifyingKey::from_bytes(&some!(self.sign_pk)?)?,
            sign_sk: SigningKey::from_bytes(&some!(self.sign_sk)?),
            timeout: self.timeout,
            data_timeout: self.data_timeout,
            ttl: self.ttl,
            trusted_ip_count: 0,
            max_stream_bytes: self.max_stream_bytes,
            max_stream_duration: self.max_stream_duration,
            max_waiting_pullers: self.max_waiting_pullers,
        };
        Ok(Client::new(conf))
    }

    /// Build a [`Server`] from the builder.
    ///
    /// # Errors
    ///
    /// If any fields required for a server were not provided,
    /// [`Error::MissingField`] is returned.
    pub fn build_server(self) -> Result<Server, Error> {
        let max_clients = self.max_clients.map_or(10, NonZeroUsize::get);
        let trusted_ip_count = match max_clients / 10 {
            0 => 1,
            n => n,
        };
        let conf = Config {
            connect: crate::DEFAULT_CONNECT,
            listen: self.addr.unwrap_or(crate::DEFAULT_LISTEN),
            max_len: self.max_len.map_or(0, |x| x << 20),
            max_clients,
            encrypt_sk: [0; 32],
            encrypt_sk_id: 0,
            psk: some!(self.psk)?,
            sign_pk: VerifyingKey::from_bytes(&some!(self.sign_pk)?)?,
            sign_sk: SigningKey::from_bytes(&[0; 32]),
            timeout: self.timeout,
            data_timeout: self.timeout,
            ttl: self.ttl,
            trusted_ip_count,
            max_stream_bytes: self.max_stream_bytes,
            max_stream_duration: self.max_stream_duration,
            max_waiting_pullers: self.max_waiting_pullers,
        };
        Ok(Server::new(conf))
    }
}

impl core::fmt::Debug for Builder {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let display_secrets = f.alternate();
        let mut s = f.debug_struct("ConfigBuilder");
        s.field("addr", &self.addr)
            .field("max_clients", &self.max_clients)
            .field("max_len", &self.max_len)
            .field("timeout", &self.timeout)
            .field("data_timeout", &self.data_timeout)
            .field("ttl", &self.ttl)
            .field("max_stream_bytes", &self.max_stream_bytes)
            .field("max_stream_duration", &self.max_stream_duration)
            .field("max_waiting_pullers", &self.max_waiting_pullers);
        if display_secrets {
            let mut buf = [0; 16];
            let hex = self.encrypt_sk_id.map(|x| crate::util::hex(&x.to_le_bytes(), &mut buf));
            s.field("encrypt_sk", &self.encrypt_sk)
                .field("encrypt_sk_id", &hex)
                .field("psk", &self.psk)
                .field("sign_pk", &self.sign_pk)
                .field("sign_sk", &self.sign_sk)
                .finish()
        } else {
            s.finish_non_exhaustive()
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
