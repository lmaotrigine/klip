use std::{
    borrow::Cow,
    fmt::{Debug, Display},
};

/// Represents an error that can occur in klip.
///
/// Many of the variants are deliberately opaque as they represent failure
/// states in cryptographic operations.
pub enum Error {
    /// Authentication failed
    Auth,
    /// The server has reached its maximum capacity and cannot accept any more
    /// clients
    CapacityReached,
    /// The clipboard is empty
    Empty,
    /// The client and server are running different versions of the protocol
    IncompatibleVersions {
        /// The protocol version used by the client
        client: u8,
        /// The protocol version used by the server
        server: u8,
    },
    /// A config field has an invalid value
    InvalidField(&'static str),
    /// An I/O error occurred
    Io(std::io::Error),
    /// The content requested to be stored is larger than the configured maximum
    /// size
    Large {
        /// The configured maximum size
        max: u64,
        /// The actual size of the content that was attempted to be stored
        got: u64,
    },
    /// The bytes read from the stream could not be parsed.
    MaybeIncompatibleVersion,
    /// The server closed the connection unexpectedly.
    ///
    /// This usually indicates a protocol mismatch or a decryption/verification
    /// error
    ConnectionClosed,
    /// The clipboard content is older than the configured maximum age
    Old,
    /// The server doesn't support the protocol used by the client
    ProtocolUnsupported,
    /// The secret key ID retrieved from the server does not match the one in
    /// the client's configuration.
    SecretKeyIDMismatch {
        /// The secret key ID expected by the client
        expected: u64,
        /// The secret key ID reported by the server
        actual: u64,
    },
    /// The encrypted clipboard content is too short to be valid
    ///
    /// The inner value is the actual length of the ciphertext that is only
    /// logged on the server side.
    Short(Option<u64>),
    /// Ed25519 signature verification failed
    Signature,
    /// The provided opcode is not recognised by the server
    UnknownOp,
    /// The clipboard content has a timestamp that is too far in the future
    Future,
    /// XChaCha20-Poly1305 encryption/decryption error
    Aead(chacha20poly1305::Error),
    /// A stream chunk was larger than expected
    ChunkTooLarge {
        /// The configured maximum chunk size
        max: u32,
        /// The actual size of the chunk that was attempted to be sent
        got: u32,
    },
    /// The total stream size exceeded the configured maximum size
    StreamTooLarge,
    /// The stream duration exceeded the configured maximum duration
    StreamDurationExceeded,
    /// A stream is already active
    StreamActive {
        /// Indicates whether this was returned inside a push or pull operation,
        /// for display purposes
        push: bool,
    },
    /// Too many clients are waiting to pull the stream
    TooManyPullers,
    /// A puller timed out while waiting for the stream to become available
    PullerTimeout(u64),
    /// No clients are waiting to receive the stream
    NoWaiters,
    /// The server rejected the stream
    GenericStream,
    /// Another error occurred
    Message(Cow<'static, str>),
}

impl Error {
    pub(crate) fn message<S: Into<Cow<'static, str>>>(s: S) -> Self {
        Self::Message(s.into())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Auth => f.write_str("authentication failed"),
            Self::CapacityReached => f.write_str("cannot accept any more clients"),
            Self::Empty => f.write_str("the clipboard may be empty"),
            Self::IncompatibleVersions { client, server } => {
                write!(f, "incompatible server version (client: {client}, server: {server})")
            }
            Self::InvalidField(field) => write!(f, "invalid value for config field `{field}`"),
            Self::Io(e) => Display::fmt(e, f),
            Self::Large { max, got } => write!(
                f,
                "{got} bytes requested to be stored, but limit set to {max} bytes ({} MiB)",
                max << 20
            ),
            Self::MaybeIncompatibleVersion => {
                f.write_str("the server may be running an incompatible version")
            }
            Self::ConnectionClosed => f.write_str(
                "the server unexpectedly closed the connection. it may be running an incompatible \
                 version.\ncheck the server logs for more information.",
            ),
            Self::Old => f.write_str("the clipboard content is too old"),
            Self::ProtocolUnsupported => f.write_str("the server doesn't support this protocol"),
            Self::SecretKeyIDMismatch { expected, actual } => write!(
                f,
                "configured key ID is {expected:x}, but content was encrypted using key ID \
                 {actual:x}"
            ),
            Self::Short(None) => f.write_str("the clipboard content is too short"),
            Self::Short(Some(len)) => write!(f, "short encrypted message (only {len} bytes)"),
            Self::Signature => f.write_str("signature verification failed"),
            Self::UnknownOp => f.write_str("unknown opcode"),
            Self::Future => f.write_str("clipboard content timestamp is too far in the future"),
            Self::Aead(e) => write!(f, "{e}"),
            Self::ChunkTooLarge { max, got } => {
                write!(f, "stream: chunk too large ({got} > {max})")
            }
            Self::StreamTooLarge => f.write_str("stream rejected: exceeded maximum stream size"),
            Self::StreamDurationExceeded => {
                f.write_str("stream rejected: exceeded maximum stream duration")
            }
            Self::StreamActive { push } => write!(
                f,
                "stream {} rejected: stream already active",
                if *push { "push" } else { "pull" }
            ),
            Self::TooManyPullers => f.write_str("stream pull rejected: too many waiting pullers"),
            Self::PullerTimeout(id) => write!(f, "puller {id}: wait timeout expired"),
            Self::NoWaiters => f.write_str("no clients are waiting to receive the stream"),
            Self::GenericStream => f.write_str("the server rejected the stream"),
            Self::Message(s) => f.write_str(s),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Aead(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ed25519_dalek::SignatureError> for Error {
    fn from(_: ed25519_dalek::SignatureError) -> Self {
        Self::Signature
    }
}

impl From<chacha20poly1305::Error> for Error {
    fn from(value: chacha20poly1305::Error) -> Self {
        Self::Aead(value)
    }
}
