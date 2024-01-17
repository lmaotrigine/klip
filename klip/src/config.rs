use crate::{
    cli::{Cli, Command},
    error::Error,
};
use ed25519::{SigningKey, VerifyingKey};
use std::{net::SocketAddr, time::Duration};

#[allow(clippy::module_name_repetitions)]
pub struct TomlConfig {
    table: toml::value::Table,
}

impl TomlConfig {
    pub const fn new(table: toml::value::Table) -> Self {
        Self { table }
    }

    pub fn connect(&self) -> SocketAddr {
        if let Some(toml::Value::String(v)) = self.table.get("connect") {
            v.parse().unwrap_or(crate::DEFAULT_CONNECT)
        } else {
            crate::DEFAULT_CONNECT
        }
    }

    pub fn listen(&self) -> SocketAddr {
        if let Some(toml::Value::String(v)) = self.table.get("listen") {
            v.parse().unwrap_or(crate::DEFAULT_LISTEN)
        } else {
            crate::DEFAULT_LISTEN
        }
    }

    pub fn encrypt_sk(&self) -> Result<[u8; 32], Error> {
        if let Some(toml::Value::String(v)) = self.table.get("encrypt_sk") {
            let mut buf = [0; 32];
            crate::util::from_hex(v, &mut buf).map_err(|()| Error::InvalidField("encrypt_sk"))?;
            Ok(buf)
        } else {
            Err(Error::MissingField("encrypt_sk"))
        }
    }

    pub fn encrypt_sk_id(&self) -> Result<u64, Error> {
        let mut buf = [0; 8];
        if let Some(toml::Value::String(v)) = self.table.get("encrypt_sk_id") {
            crate::util::from_hex(v, &mut buf)
                .map_err(|()| Error::InvalidField("encrypt_sk_id"))?;
        } else {
            let encrypt_sk = self.encrypt_sk()?;
            let mut hasher =
                blake2b::Blake2b::new_with_params(&[], crate::DOMAIN.as_bytes(), &[], 8);
            hasher.update(&encrypt_sk);
            buf.copy_from_slice(hasher.finalize().as_bytes());
        }
        Ok(u64::from_le_bytes(buf))
    }

    pub fn psk(&self) -> Result<[u8; 32], Error> {
        if let Some(toml::Value::String(v)) = self.table.get("psk") {
            let mut buf = [0; 32];
            crate::util::from_hex(v, &mut buf).map_err(|()| Error::InvalidField("psk"))?;
            Ok(buf)
        } else {
            Err(Error::MissingField("psk"))
        }
    }

    pub fn sign_pk(&self) -> Result<VerifyingKey, Error> {
        if let Some(toml::Value::String(v)) = self.table.get("sign_pk") {
            let mut buf = [0; 32];
            crate::util::from_hex(v, &mut buf).map_err(|()| Error::InvalidField("sign_pk"))?;
            VerifyingKey::from_bytes(&buf).map_err(|_| Error::InvalidField("sign_pk"))
        } else {
            Err(Error::MissingField("sign_pk"))
        }
    }

    pub fn sign_sk(&self) -> Result<SigningKey, Error> {
        if let Some(toml::Value::String(v)) = self.table.get("sign_sk") {
            let mut buf = [0; 32];
            crate::util::from_hex(v, &mut buf).map_err(|()| Error::InvalidField("sign_sk"))?;
            Ok(SigningKey::from_bytes(&buf))
        } else {
            Err(Error::MissingField("sign_sk"))
        }
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn ttl(&self) -> Duration {
        if let Some(toml::Value::Integer(v)) = self.table.get("ttl") {
            if *v > 0 {
                Duration::from_secs(*v as u64)
            } else {
                crate::DEFAULT_TTL
            }
        } else {
            crate::DEFAULT_TTL
        }
    }
}

pub struct Config {
    connect: SocketAddr,
    listen: SocketAddr,
    max_clients: usize,
    max_len: u64,
    encrypt_sk: [u8; 32],
    encrypt_sk_id: u64,
    psk: [u8; 32],
    sign_pk: VerifyingKey,
    sign_sk: SigningKey,
    timeout: Duration,
    data_timeout: Duration,
    ttl: Duration,
    trusted_ip_count: usize,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_secrets = f.alternate();
        let mut s = f.debug_struct("Config");
        s.field("connect", &self.connect)
            .field("listen", &self.listen)
            .field("max_clients", &self.max_clients)
            .field("max_len", &self.max_len)
            .field("timeout", &self.timeout)
            .field("data_timeout", &self.data_timeout)
            .field("ttl", &self.ttl)
            .field("trusted_ip_count", &self.trusted_ip_count);
        if display_secrets {
            s.field("encrypt_sk", &self.encrypt_sk);
            let mut out = [0; 16];
            let inp = self.encrypt_sk_id.to_le_bytes();
            crate::util::hex(&inp, &mut out);
            s.field("encrypt_sk_id", &unsafe {
                std::str::from_utf8_unchecked(&out)
            });
            s.field("psk", &self.psk);
            s.field("sign_pk", &self.sign_pk);
            s.field("sign_sk", &self.sign_sk);
            s.finish()
        } else {
            s.finish_non_exhaustive()
        }
    }
}

impl Config {
    pub fn new(t: &TomlConfig, c: &Cli) -> Result<Self, Error> {
        Ok(Self {
            connect: t.connect(),
            listen: t.listen(),
            max_len: if let Command::Serve(args) = c.subcommand {
                args.max_len_mb * 1024 * 1024
            } else {
                1
            },
            max_clients: if let Command::Serve(args) = c.subcommand {
                args.max_clients.get()
            } else {
                1
            },
            encrypt_sk: if let Command::Serve(_) = c.subcommand {
                [0; 32]
            } else {
                t.encrypt_sk()?
            },
            encrypt_sk_id: if let Command::Serve(_) = c.subcommand {
                0
            } else {
                t.encrypt_sk_id()?
            },
            psk: t.psk()?,
            sign_pk: t.sign_pk()?,
            sign_sk: if let Command::Serve(_) = c.subcommand {
                SigningKey::from_bytes(&[0; 32])
            } else {
                t.sign_sk()?
            },
            timeout: if let Command::Serve(args) = c.subcommand {
                Duration::from_secs(args.timeout)
            } else {
                Duration::from_secs(10)
            },
            data_timeout: if let Command::Serve(args) = c.subcommand {
                Duration::from_secs(args.data_timeout)
            } else {
                Duration::from_secs(3600)
            },
            ttl: t.ttl(),
            trusted_ip_count: if let Command::Serve(args) = c.subcommand {
                match args.max_clients.get() / 10 {
                    0 => 1,
                    n => n,
                }
            } else {
                0
            },
        })
    }

    pub const fn psk(&self) -> [u8; 32] {
        self.psk
    }

    pub const fn data_timeout(&self) -> Duration {
        self.data_timeout
    }

    pub const fn max_len(&self) -> u64 {
        self.max_len
    }

    pub const fn trusted_ip_count(&self) -> usize {
        self.trusted_ip_count
    }

    pub const fn timeout(&self) -> Duration {
        self.timeout
    }

    pub const fn max_clients(&self) -> usize {
        self.max_clients
    }

    pub const fn listen(&self) -> SocketAddr {
        self.listen
    }

    pub const fn encrypt_sk_id(&self) -> u64 {
        self.encrypt_sk_id
    }

    pub const fn connect(&self) -> SocketAddr {
        self.connect
    }

    pub const fn encrypt_sk(&self) -> [u8; 32] {
        self.encrypt_sk
    }

    pub const fn ttl(&self) -> Duration {
        self.ttl
    }

    pub const fn sign_pk(&self) -> VerifyingKey {
        self.sign_pk
    }

    pub const fn sign_sk(&self) -> &SigningKey {
        &self.sign_sk
    }
}
