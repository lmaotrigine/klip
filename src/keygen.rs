use crate::util::hex;
use rand::{Rng, TryRng};

struct DeterministicRandom {
    pool: [u8; 96],
    pos: usize,
}

impl DeterministicRandom {
    pub fn init(key: &[u8]) -> Self {
        let mut out = [0; 96];
        scrypt::scrypt(
            key,
            &[],
            &scrypt::Params::new(14, 12, 1)
                .expect("invalid scrypt params were passed. this is a bug."),
            &mut out,
        )
        .expect("scrypt failed. this is a bug.");
        Self { pool: out, pos: 0 }
    }
}

impl TryRng for DeterministicRandom {
    type Error = core::convert::Infallible;

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        let req_len = dest.len();
        dest.copy_from_slice(&self.pool[self.pos..self.pos + req_len]);
        for b in self.pool.iter_mut().take(req_len) {
            *b = 0;
        }
        self.pos += req_len;
        Ok(())
    }

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        let mut buf = [0; 4];
        self.fill_bytes(&mut buf);
        Ok(u32::from_le_bytes(buf))
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let mut buf = [0; 8];
        self.fill_bytes(&mut buf);
        Ok(u64::from_le_bytes(buf))
    }
}

enum Rand {
    OsRng(rand::rngs::ThreadRng),
    Deterministic(DeterministicRandom),
}

impl TryRng for Rand {
    type Error = core::convert::Infallible;

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        match self {
            Self::OsRng(rng) => rng.try_fill_bytes(dest),
            Self::Deterministic(rng) => rng.try_fill_bytes(dest),
        }
    }

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        match self {
            Self::OsRng(rng) => rng.try_next_u32(),
            Self::Deterministic(rng) => rng.try_next_u32(),
        }
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        match self {
            Self::OsRng(rng) => rng.try_next_u64(),
            Self::Deterministic(rng) => rng.try_next_u64(),
        }
    }
}

/// A set of keys required by the klip protocol.
#[derive(Clone, Copy)]
pub struct Keys {
    psk: [u8; 32],
    sign_pk: [u8; 32],
    sign_sk: [u8; 32],
    encrypt_sk: [u8; 32],
}

impl Keys {
    /// Generate a new set of keys. If `passphrase` is `Some`, the keys will
    /// be derived deterministically from the passphrase.
    ///
    /// It is recommended to generate random keys by passing `None` unless you
    /// have a specific reason such as first time setup on some airgapped
    /// systems. Even so, it is recommended to replace the keys with random ones
    /// as soon as possible.
    pub fn generate<S: AsRef<[u8]>>(passphrase: Option<S>) -> Self {
        let mut rng = passphrase.map_or_else(
            || Rand::OsRng(rand::rng()),
            |passphrase| Rand::Deterministic(DeterministicRandom::init(passphrase.as_ref())),
        );
        let mut psk = [0; 32];
        rng.fill_bytes(&mut psk);
        let mut sign_sk = [0; 32];
        rng.fill_bytes(&mut sign_sk);
        let mut encrypt_sk = [0; 32];
        rng.fill_bytes(&mut encrypt_sk);
        let sign_sk = ed25519_dalek::SigningKey::from_bytes(&sign_sk);
        Self {
            psk,
            sign_pk: sign_sk.verifying_key().to_bytes(),
            sign_sk: sign_sk.to_bytes(),
            encrypt_sk,
        }
    }

    /// The 256-bit pre-shared key used for authentication. This is required for
    /// both clients and servers.
    #[must_use]
    pub const fn psk(&self) -> [u8; 32] {
        self.psk
    }

    /// The Ed25519 public key used for verifying signatures. This is required
    /// for both clients and servers.
    #[must_use]
    pub const fn sign_pk(&self) -> [u8; 32] {
        self.sign_pk
    }

    /// The Ed25519 private key used for signing messages. This is required for
    /// clients, and should preferably not be exposed to the server.
    #[must_use]
    pub const fn sign_sk(&self) -> [u8; 32] {
        self.sign_sk
    }

    /// The 256-bit private key used for encryption. This is required for
    /// clients, and should preferably not be exposed to the server.
    #[must_use]
    pub const fn encrypt_sk(&self) -> [u8; 32] {
        self.encrypt_sk
    }
}

impl core::fmt::Debug for Keys {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut buf = [0; 64];
        if f.alternate() {
            f.debug_struct("Keys")
                .field("psk", &hex(&self.psk, &mut buf))
                .field("sign_pk", &hex(&self.sign_pk, &mut buf))
                .field("sign_sk", &hex(&self.sign_sk, &mut buf))
                .field("encrypt_sk", &hex(&self.encrypt_sk, &mut buf))
                .finish()
        } else {
            f.debug_struct("Keys").finish_non_exhaustive()
        }
    }
}

#[cfg(feature = "cli")]
pub fn generate_keys(config_file_name: impl core::fmt::Display, key: &[u8]) {
    let keys = Keys::generate((!key.is_empty()).then_some(key));
    let mut psk_hex_buf = [0; 64];
    let psk = hex(&keys.psk, &mut psk_hex_buf);
    let mut encrypt_sk_hex_buf = [0; 64];
    let encrypt_sk = hex(&keys.encrypt_sk, &mut encrypt_sk_hex_buf);
    let mut signing_key_hex = [0; 64];
    let signing_key = hex(&keys.sign_sk, &mut signing_key_hex);
    let mut verifying_key_hex = [0; 64];
    let verifying_key = hex(&keys.sign_pk, &mut verifying_key_hex);
    println!(
        "\n\n--- Create a file named {config_file_name} with only the lines relevant to your \
         configuration ---\n\n"
    );
    println!("# Configuration for a client\n");
    println!("connect    = \"{}\"\t# edit appropriately", crate::DEFAULT_CONNECT);
    println!("psk        = \"{psk}\"");
    println!("sign_pk    = \"{verifying_key}\"");
    println!("sign_sk    = \"{signing_key}\"");
    println!("encrypt_sk = \"{encrypt_sk}\"");
    println!();
    println!("# Configuration for a server\n");
    println!("listen     = \"{}\"\t# edit appropriately", crate::DEFAULT_LISTEN);
    println!("psk        = \"{psk}\"");
    println!("sign_pk    = \"{verifying_key}\"");
    println!();
    println!("# Hybrid configuration\n");
    println!("connect    = \"{}\"\t# edit appropriately", crate::DEFAULT_CONNECT);
    println!("listen     = \"{}\"\t# edit appropriately", crate::DEFAULT_LISTEN);
    println!("psk        = \"{psk}\"");
    println!("sign_pk    = \"{verifying_key}\"");
    println!("sign_sk    = \"{signing_key}\"");
    println!("encrypt_sk = \"{encrypt_sk}\"");
}
