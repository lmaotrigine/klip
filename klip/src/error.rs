use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

pub enum Error {
    Auth,
    CapacityReached,
    Empty,
    IncompatibleVersions { client: u8, server: u8 },
    InvalidField(&'static str),
    Io(std::io::Error),
    Large { max: u64, got: u64 },
    MaybeIncompatibleVersion,
    MissingField(&'static str),
    NoHome,
    Old,
    ProtocolUnsupported,
    SecretKeyIDMismatch { expected: u64, actual: u64 },
    Short,
    ShortCiphertext(u64),
    Signature,
    Toml(toml::de::Error),
    UnknownOp,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auth => f.write_str("authentication failed"),
            Self::CapacityReached => f.write_str("cannot accept any more clients"),
            Self::Empty => f.write_str("the clipboard may be empty"),
            Self::IncompatibleVersions { client, server } => write!(
                f,
                "incompatible server version (client: {client}, server: {server})"
            ),
            Self::InvalidField(field) => write!(f, "invalid value for config field `{field}`"),
            Self::Io(e) => Display::fmt(e, f),
            Self::Large { max, got } => write!(
                f,
                "{got} bytes requested to be stored, but limit set to {max} bytes ({} MiB)",
                max / (1024 * 1024)
            ),
            Self::MaybeIncompatibleVersion => {
                f.write_str("the server may be running an incompatible version")
            }
            Self::MissingField(field) => write!(f, "missing required config field `{field}`"),
            Self::NoHome => f.write_str("could not determine home directory"),
            Self::Old => f.write_str("the clipboard content is too old"),
            Self::ProtocolUnsupported => f.write_str("the server doesn't support this protocol"),
            Self::SecretKeyIDMismatch { expected, actual } => write!(
                f,
                "configured key ID is {expected:x}, but content was encrypted using key ID \
                 {actual:x}"
            ),
            Self::Short => f.write_str("the clipboard content is too short"),
            Self::ShortCiphertext(len) => write!(f, "short encrypted message (only {len} bytes)"),
            Self::Signature => f.write_str("signature verification failed"),
            Self::Toml(e) => write!(f, "could not parse TOML config: {e}"),
            Self::UnknownOp => f.write_str("unknown opcode"),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml(value)
    }
}

impl From<ed25519::SignatureError> for Error {
    fn from(_: ed25519::SignatureError) -> Self {
        Self::Signature
    }
}

pub struct Context {
    error: Error,
    context: Option<Cow<'static, str>>,
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(context) = &self.context {
            write!(f, "{}: {}", context, self.error)
        } else {
            Display::fmt(&self.error, f)
        }
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl From<Error> for Context {
    fn from(error: Error) -> Self {
        Self {
            error,
            context: None,
        }
    }
}

pub trait ResultExt<T> {
    fn context<C: Into<Cow<'static, str>>>(self, context: C) -> Result<T, Context>;
}

impl<T, E: Into<Error>> ResultExt<T> for Result<T, E> {
    fn context<C: Into<Cow<'static, str>>>(self, context: C) -> Result<T, Context> {
        self.map_err(|e| Context {
            error: e.into(),
            context: Some(context.into()),
        })
    }
}
