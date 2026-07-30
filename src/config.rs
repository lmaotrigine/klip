use crate::{
    cli::{Cli, Subcommand},
    error::Error,
};
use blake2::digest::{Mac, typenum::U8};
use ed25519_dalek::{SigningKey, VerifyingKey};
use std::{net::SocketAddr, time::Duration};

struct TomlConf {
    table: toml::Table,
}

impl TomlConf {
    const fn new(table: toml::Table) -> Self {
        Self { table }
    }

    fn connect(&self) -> Result<SocketAddr, Error> {
        if let Some(toml::Value::String(v)) = self.table.get("connect") {
            v.parse().map_err(|_| Error::InvalidField("connect"))
        } else {
            Ok(crate::DEFAULT_CONNECT)
        }
    }

    fn listen(&self) -> Result<SocketAddr, Error> {
        if let Some(toml::Value::String(v)) = self.table.get("listen") {
            v.parse().map_err(|_| Error::InvalidField("listen"))
        } else {
            Ok(crate::DEFAULT_LISTEN)
        }
    }

    fn encrypt_sk(&self) -> Result<[u8; 32], Error> {
        if let Some(toml::Value::String(v)) = self.table.get("encrypt_sk") {
            let mut buf = [0; 32];
            crate::util::from_hex(v, &mut buf).map_err(|()| Error::InvalidField("encrypt_sk"))?;
            Ok(buf)
        } else {
            Err(Error::MissingField("encrypt_sk"))
        }
    }

    fn encrypt_sk_id(&self) -> Result<u64, Error> {
        let mut buf = [0; 8];
        if let Some(toml::Value::String(v)) = self.table.get("encrypt_sk_id") {
            crate::util::from_hex(v, &mut buf)
                .map_err(|()| Error::InvalidField("encrypt_sk_id"))?;
        } else {
            let encrypt_sk = self.encrypt_sk()?;
            let mut hasher = blake2::Blake2bMac::<U8>::new_with_salt_and_personal(
                None,
                &[],
                crate::DOMAIN.as_bytes(),
            )
            .expect("invalid params in blake2b");
            hasher.update(&encrypt_sk);
            buf.copy_from_slice(&hasher.finalize().into_bytes());
        }
        Ok(u64::from_le_bytes(buf))
    }

    fn psk(&self) -> Result<[u8; 32], Error> {
        if let Some(toml::Value::String(v)) = self.table.get("psk") {
            let mut buf = [0; 32];
            crate::util::from_hex(v, &mut buf).map_err(|()| Error::InvalidField("psk"))?;
            Ok(buf)
        } else {
            Err(Error::MissingField("psk"))
        }
    }

    fn sign_pk(&self) -> Result<VerifyingKey, Error> {
        if let Some(toml::Value::String(v)) = self.table.get("sign_pk") {
            let mut buf = [0; 32];
            crate::util::from_hex(v, &mut buf).map_err(|()| Error::InvalidField("sign_pk"))?;
            VerifyingKey::from_bytes(&buf).map_err(|_| Error::InvalidField("sign_pk"))
        } else {
            Err(Error::MissingField("sign_pk"))
        }
    }

    fn sign_sk(&self) -> Result<SigningKey, Error> {
        if let Some(toml::Value::String(v)) = self.table.get("sign_sk") {
            let mut buf = [0; 32];
            crate::util::from_hex(v, &mut buf).map_err(|()| Error::InvalidField("sign_sk"))?;
            Ok(SigningKey::from_bytes(&buf))
        } else {
            Err(Error::MissingField("sign_sk"))
        }
    }

    #[expect(clippy::cast_sign_loss)]
    fn ttl(&self) -> Duration {
        if let Some(toml::Value::Integer(v)) = self.table.get("ttl") {
            if *v > 0 { Duration::from_secs(*v as u64) } else { crate::DEFAULT_TTL }
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
            s.field("encrypt_sk_id", &std::str::from_utf8(&out).expect("hex should be valid utf8"));
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
    pub fn new(t: toml::Table, c: &Cli) -> Result<Self, Error> {
        let t = TomlConf::new(t);
        Ok(Self {
            connect: t.connect()?,
            listen: t.listen()?,
            max_len: if let Subcommand::Serve(args) = c.subcommand {
                args.max_len_mb * 1024 * 1024
            } else {
                1
            },
            max_clients: if let Subcommand::Serve(args) = c.subcommand {
                args.max_clients.get()
            } else {
                1
            },
            encrypt_sk: if let Subcommand::Serve(_) = c.subcommand {
                [0; 32]
            } else {
                t.encrypt_sk()?
            },
            encrypt_sk_id: if let Subcommand::Serve(_) = c.subcommand {
                0
            } else {
                t.encrypt_sk_id()?
            },
            psk: t.psk()?,
            sign_pk: t.sign_pk()?,
            sign_sk: if let Subcommand::Serve(_) = c.subcommand {
                SigningKey::from_bytes(&[0; 32])
            } else {
                t.sign_sk()?
            },
            timeout: if let Subcommand::Serve(args) = c.subcommand {
                Duration::from_secs(args.timeout)
            } else {
                Duration::from_secs(10)
            },
            data_timeout: if let Subcommand::Serve(args) = c.subcommand {
                Duration::from_secs(args.data_timeout)
            } else {
                Duration::from_hours(1)
            },
            ttl: t.ttl(),
            trusted_ip_count: if let Subcommand::Serve(args) = c.subcommand {
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
