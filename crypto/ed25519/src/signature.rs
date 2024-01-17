use crate::{
    consts::SIGNATURE_LENGTH,
    errors::{InternalError, SignatureError},
};
use core::fmt::Debug;
use curve25519::{CompressedEdwardsY, Scalar};

#[derive(Copy, PartialEq, Eq)]
pub struct Signature {
    pub(crate) r: CompressedEdwardsY,
    pub(crate) s: Scalar,
}

#[allow(clippy::expl_impl_clone_on_copy)]
impl Clone for Signature {
    fn clone(&self) -> Self {
        *self
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Signature( R: {:?}, s: {:?} )", self.r, self.s)
    }
}

#[inline(always)]
fn check_scalar(bytes: [u8; 32]) -> Result<Scalar, SignatureError> {
    Option::from(Scalar::from_canonical_bytes(bytes))
        .ok_or_else(|| InternalError::ScalarFormat.into())
}

impl Signature {
    #[inline]
    #[must_use]
    pub fn to_bytes(&self) -> [u8; SIGNATURE_LENGTH] {
        let mut signature_bytes = [0; SIGNATURE_LENGTH];
        signature_bytes[..32].copy_from_slice(&self.r.as_bytes()[..]);
        signature_bytes[32..].copy_from_slice(&self.s.as_bytes()[..]);
        signature_bytes
    }

    #[inline]
    pub fn from_bytes(bytes: &[u8; SIGNATURE_LENGTH]) -> Result<Self, SignatureError> {
        let mut r_bytes = [0; 32];
        let mut s_bytes = [0; 32];
        r_bytes.copy_from_slice(&bytes[..32]);
        s_bytes.copy_from_slice(&bytes[32..]);
        Ok(Self {
            r: CompressedEdwardsY(r_bytes),
            s: check_scalar(s_bytes)?,
        })
    }
}
