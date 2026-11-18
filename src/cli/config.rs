use crate::{
    cli::{Cli, Subcommand},
    error::Error,
};
use blake2::digest::{Mac, typenum::U8};
use core::{net::SocketAddr, time::Duration};
use ed25519_dalek::{SigningKey, VerifyingKey};

pub struct Config<'a> {
    table: toml::Table,
    cli: &'a Cli,
}

impl<'a> Config<'a> {
    pub const fn new(table: toml::Table, cli: &'a Cli) -> Self {
        Self { table, cli }
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
        if self.serve_args().is_some() {
            return Ok([0; 32]);
        }
        if let Some(toml::Value::String(v)) = self.table.get("encrypt_sk") {
            let mut buf = [0; 32];
            crate::util::from_hex(v, &mut buf).map_err(|()| Error::InvalidField("encrypt_sk"))?;
            Ok(buf)
        } else {
            Err(Error::MissingField("encrypt_sk"))
        }
    }

    const fn serve_args(&self) -> Option<&crate::cli::ServerArgs> {
        if let Subcommand::Serve(args) = &self.cli.subcommand { Some(args) } else { None }
    }

    fn max_len(&self) -> u64 {
        self.serve_args().map_or(1, |a| a.max_len_mb << 20)
    }

    fn max_clients(&self) -> usize {
        self.serve_args().map_or(1, |a| a.max_clients.get())
    }

    fn encrypt_sk_id(&self) -> Result<u64, Error> {
        if self.serve_args().is_some() {
            return Ok(0);
        }
        let encrypt_sk = self.encrypt_sk()?;
        let mut hasher = blake2::Blake2bMac::<U8>::new_with_salt_and_personal(
            None,
            &[],
            crate::DOMAIN.as_bytes(),
        )
        .expect("invalid params in blake2b");
        hasher.update(&encrypt_sk);
        Ok(u64::from_le_bytes(hasher.finalize().into_bytes().into()))
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
        if self.serve_args().is_some() {
            return Ok(SigningKey::from_bytes(&[0; 32]));
        }
        if let Some(toml::Value::String(v)) = self.table.get("sign_sk") {
            let mut buf = [0; 32];
            crate::util::from_hex(v, &mut buf).map_err(|()| Error::InvalidField("sign_sk"))?;
            Ok(SigningKey::from_bytes(&buf))
        } else {
            Err(Error::MissingField("sign_sk"))
        }
    }

    const fn timeout(&self) -> Duration {
        match self.cli.subcommand {
            Subcommand::Copy(args)
            | Subcommand::Move(args)
            | Subcommand::Paste(args)
            | Subcommand::Push(args, _)
            | Subcommand::Pull(args, _) => Duration::from_secs(args.timeout),
            Subcommand::Serve(args) => Duration::from_secs(args.timeout),
            _ => Duration::from_secs(10),
        }
    }

    const fn data_timeout(&self) -> Duration {
        match self.cli.subcommand {
            Subcommand::Copy(args)
            | Subcommand::Move(args)
            | Subcommand::Paste(args)
            | Subcommand::Push(args, _)
            | Subcommand::Pull(args, _) => Duration::from_secs(args.data_timeout),
            Subcommand::Serve(args) => Duration::from_secs(args.data_timeout),
            _ => Duration::from_hours(1),
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

    #[expect(clippy::cast_sign_loss)]
    fn max_stream_bytes(&self) -> u64 {
        if let Some(toml::Value::Integer(v)) = self.table.get("max_stream_bytes") {
            if *v > 0 {
                (*v as u64).max(crate::DEFAULT_MAX_STREAM_BYTES)
            } else {
                crate::DEFAULT_MAX_STREAM_BYTES
            }
        } else {
            crate::DEFAULT_MAX_STREAM_BYTES
        }
    }

    #[expect(clippy::cast_sign_loss)]
    fn max_stream_duration(&self) -> Duration {
        if let Some(toml::Value::Integer(v)) = self.table.get("max_stream_duration") {
            if *v > 0 {
                Duration::from_secs(*v as u64).max(crate::DEFAULT_MAX_STREAM_DURATION)
            } else {
                crate::DEFAULT_MAX_STREAM_DURATION
            }
        } else {
            crate::DEFAULT_MAX_STREAM_DURATION
        }
    }

    fn max_waiting_pullers(&self) -> usize {
        if let Some(toml::Value::Integer(v)) = self.table.get("max_waiting_pullers") {
            if *v > 0 {
                usize::try_from(*v).unwrap_or(crate::DEFAULT_MAX_WAITING_PULLERS)
            } else {
                crate::DEFAULT_MAX_WAITING_PULLERS
            }
        } else {
            crate::DEFAULT_MAX_WAITING_PULLERS
        }
    }

    pub fn into_client(self) -> Result<crate::Client, Error> {
        Ok(crate::Client {
            addr: self.connect()?,
            encrypt_sk: self.encrypt_sk()?,
            encrypt_sk_id: self.encrypt_sk_id()?,
            psk: self.psk()?,
            sign_pk: self.sign_pk()?,
            sign_sk: self.sign_sk()?,
            timeout: self.timeout(),
            data_timeout: self.data_timeout(),
            ttl: self.ttl(),
            max_stream_bytes: self.max_stream_bytes(),
            max_stream_duration: self.max_stream_duration(),
        })
    }

    pub fn into_server(self) -> Result<crate::Server, Error> {
        Ok(crate::Server {
            addr: self.listen()?,
            max_clients: self.max_clients(),
            psk: self.psk()?,
            sign_pk: self.sign_pk()?,
            timeout: self.timeout(),
            data_timeout: self.data_timeout(),
            ttl: self.ttl(),
            max_len: self.max_len(),
            max_stream_bytes: self.max_stream_bytes(),
            max_stream_duration: self.max_stream_duration(),
            max_waiting_pullers: self.max_waiting_pullers(),
        })
    }
}
