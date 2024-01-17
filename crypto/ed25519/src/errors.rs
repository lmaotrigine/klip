use core::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalError {
    PointDecompression,
    ScalarFormat,
    BytesLength { name: &'static str, length: usize },
    Verify,
    MismatchedKeypair,
}

impl Display for InternalError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::PointDecompression => write!(f, "cannot decompress Edwards point"),
            Self::ScalarFormat => write!(f, "cannot use scalar with high-bit set"),
            Self::BytesLength { name: n, length: l } => {
                write!(f, "{n} must be {l} bytes in length")
            }
            Self::Verify => write!(f, "verification equation was not satisfied"),
            Self::MismatchedKeypair => write!(f, "mismatched keypair detected"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SignatureError(pub(crate) InternalError);

impl From<InternalError> for SignatureError {
    fn from(value: InternalError) -> Self {
        Self(value)
    }
}

impl Display for SignatureError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
