use crate::{Signature, VerifyingKey};
use crypto_common::erase::{Erase, EraseOnDrop};
use curve25519::{scalar::clamp_integer, EdwardsPoint, Scalar};
use sha512::Sha512;

#[derive(Debug)]
pub struct ExpandedSecretKey {
    pub scalar: Scalar,
    pub hash_prefix: [u8; 32],
}

impl Drop for ExpandedSecretKey {
    fn drop(&mut self) {
        self.scalar.erase();
        self.hash_prefix.erase();
    }
}

impl EraseOnDrop for ExpandedSecretKey {}

impl ExpandedSecretKey {
    pub fn from_bytes(bytes: &[u8; 64]) -> Self {
        let mut scalar_bytes = [0; 32];
        let mut hash_prefix = [0; 32];
        scalar_bytes.copy_from_slice(&bytes[..32]);
        hash_prefix.copy_from_slice(&bytes[32..]);
        let scalar = Scalar::from_bytes_mod_order(clamp_integer(scalar_bytes));
        Self {
            scalar,
            hash_prefix,
        }
    }

    pub(crate) fn raw_sign(&self, message: &[u8], verifying_key: &VerifyingKey) -> Signature {
        let mut h = Sha512::new();
        h.update(&self.hash_prefix);
        h.update(message);
        let r = Scalar::from_hash(h);
        let big_r = EdwardsPoint::mul_base(&r).compress();
        h = Sha512::new();
        h.update(big_r.as_bytes());
        h.update(verifying_key.as_bytes());
        h.update(message);
        let k = Scalar::from_hash(h);
        let s = (k * self.scalar) + r;
        Signature { r: big_r, s }
    }
}
