use crate::{
    consts::{KEYPAIR_LENGTH, SECRET_KEY_LENGTH},
    errors::{InternalError, SignatureError},
    hazmat::ExpandedSecretKey,
    Signature, VerifyingKey,
};
use core::fmt::Debug;
use crypto_common::{
    constant_time::ConstantTimeEq,
    erase::{Erase, EraseOnDrop},
};
#[cfg(any(test, feature = "rand"))]
use rand_core::CryptoRngCore;
use sha512::Sha512;

pub type SecretKey = [u8; SECRET_KEY_LENGTH];

#[derive(Clone)]
pub struct SigningKey {
    pub(crate) secret_key: SecretKey,
    pub(crate) verifying_key: VerifyingKey,
}

impl Debug for SigningKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SigningKey")
            .field("verifying_key", &self.verifying_key)
            .finish_non_exhaustive()
    }
}

impl SigningKey {
    #[inline]
    #[must_use]
    pub fn from_bytes(secret_key: &SecretKey) -> Self {
        let verifying_key = VerifyingKey::from(&ExpandedSecretKey::from(secret_key));
        Self {
            secret_key: *secret_key,
            verifying_key,
        }
    }

    #[inline]
    #[must_use]
    pub const fn to_bytes(&self) -> SecretKey {
        self.secret_key
    }

    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &SecretKey {
        &self.secret_key
    }

    #[inline]
    pub fn from_keypair_bytes(bytes: &[u8; 64]) -> Result<Self, SignatureError> {
        let (secret_key, verifying_key) = bytes.split_at(SECRET_KEY_LENGTH);
        let signing_key =
            Self::from_bytes(
                secret_key
                    .try_into()
                    .map_err(|_| InternalError::BytesLength {
                        name: "signing key",
                        length: 32,
                    })?,
            );
        let verifying_key = VerifyingKey::from_bytes(verifying_key.try_into().map_err(|_| {
            InternalError::BytesLength {
                name: "verifying key",
                length: 32,
            }
        })?)?;
        if signing_key.verifying_key() != verifying_key {
            return Err(InternalError::MismatchedKeypair.into());
        }
        Ok(signing_key)
    }

    #[must_use]
    pub fn to_keypair_bytes(&self) -> [u8; KEYPAIR_LENGTH] {
        let mut bytes = [0; KEYPAIR_LENGTH];
        bytes[..SECRET_KEY_LENGTH].copy_from_slice(&self.secret_key);
        bytes[SECRET_KEY_LENGTH..].copy_from_slice(self.verifying_key.as_bytes());
        bytes
    }

    #[must_use]
    pub const fn verifying_key(&self) -> VerifyingKey {
        self.verifying_key
    }

    #[cfg(any(test, feature = "rand"))]
    pub fn generate<R: CryptoRngCore + ?Sized>(csprng: &mut R) -> Self {
        let mut secret = SecretKey::default();
        csprng.fill_bytes(&mut secret);
        Self::from_bytes(&secret)
    }

    #[must_use]
    pub fn sign(&self, message: &[u8]) -> Signature {
        let expanded = ExpandedSecretKey::from(&self.secret_key);
        expanded.raw_sign(message, &self.verifying_key)
    }
}

impl ConstantTimeEq for SigningKey {
    fn ct_eq(&self, other: &Self) -> crypto_common::constant_time::Choice {
        self.secret_key.ct_eq(&other.secret_key)
    }
}

impl PartialEq for SigningKey {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl Eq for SigningKey {}

impl Drop for SigningKey {
    fn drop(&mut self) {
        self.secret_key.erase();
    }
}

impl EraseOnDrop for SigningKey {}

impl From<&SecretKey> for ExpandedSecretKey {
    fn from(secret_key: &SecretKey) -> Self {
        let mut hasher = Sha512::new();
        hasher.update(secret_key);
        let hash = hasher.finalize();
        Self::from_bytes(&hash)
    }
}
