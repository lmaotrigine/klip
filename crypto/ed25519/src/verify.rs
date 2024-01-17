use crate::{
    consts::PUBLIC_KEY_LENGTH,
    errors::{InternalError, SignatureError},
    hazmat::ExpandedSecretKey,
    Signature,
};
use core::{
    fmt::Debug,
    hash::{Hash, Hasher},
};
use curve25519::{CompressedEdwardsY, EdwardsPoint, Scalar};
use sha512::Sha512;

#[derive(Clone, Copy, Eq, Default)]
pub struct VerifyingKey {
    pub(crate) compressed: CompressedEdwardsY,
    pub(crate) point: EdwardsPoint,
}

impl Debug for VerifyingKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VerifyingKey({:?}, {:?})", self.compressed, self.point)
    }
}

impl Hash for VerifyingKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl PartialEq for VerifyingKey {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl VerifyingKey {
    #[inline]
    #[must_use]
    pub const fn to_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.compressed.to_bytes()
    }

    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; PUBLIC_KEY_LENGTH] {
        &(self.compressed).0
    }

    #[inline]
    pub fn from_bytes(bytes: &[u8; PUBLIC_KEY_LENGTH]) -> Result<Self, SignatureError> {
        let compressed = CompressedEdwardsY(*bytes);
        let point = compressed
            .decompress()
            .ok_or(InternalError::PointDecompression)?;
        Ok(Self { compressed, point })
    }

    fn compute_challenge(r: &CompressedEdwardsY, a: &CompressedEdwardsY, m: &[u8]) -> Scalar {
        let mut h = Sha512::new();
        h.update(r.as_bytes());
        h.update(a.as_bytes());
        h.update(m);
        Scalar::from_hash(h)
    }

    fn recompute_r(&self, signature: &Signature, m: &[u8]) -> CompressedEdwardsY {
        let k = Self::compute_challenge(&signature.r, &self.compressed, m);
        let minus_a = -self.point;
        EdwardsPoint::vartime_double_scalar_mul_basepoint(&k, &minus_a, &signature.s).compress()
    }

    pub fn verify_strict(
        &self,
        message: &[u8],
        signature: &Signature,
    ) -> Result<(), SignatureError> {
        let signature_r = signature.r.decompress().ok_or(InternalError::Verify)?;
        if signature_r.is_small_order() || self.point.is_small_order() {
            return Err(InternalError::Verify.into());
        }
        let expected_r = self.recompute_r(signature, message);
        if expected_r == signature.r {
            Ok(())
        } else {
            Err(InternalError::Verify.into())
        }
    }
}

impl From<&ExpandedSecretKey> for VerifyingKey {
    fn from(value: &ExpandedSecretKey) -> Self {
        let point = EdwardsPoint::mul_base(&value.scalar);
        Self {
            compressed: point.compress(),
            point,
        }
    }
}
