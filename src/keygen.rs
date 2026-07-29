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
        for i in 0..req_len {
            self.pool[i] = 0;
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

pub fn generate_keys(config_file_name: impl std::fmt::Display, key: &[u8]) {
    let mut rng = if key.is_empty() {
        Rand::OsRng(rand::rng())
    } else {
        Rand::Deterministic(DeterministicRandom::init(key))
    };
    let mut psk = [0; 32];
    rng.fill_bytes(&mut psk);
    let mut psk_hex = [0; 64];
    hex(&psk, &mut psk_hex);
    let mut encrypt_sk = [0; 32];
    rng.fill_bytes(&mut encrypt_sk);
    let mut encrypt_sk_hex = [0; 64];
    hex(&encrypt_sk, &mut encrypt_sk_hex);
    let mut signing_key = [0; 32];
    rng.fill_bytes(&mut signing_key);
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&signing_key);
    let verifying_key = signing_key.verifying_key();
    let mut signing_key_hex = [0; 64];
    hex(signing_key.as_bytes(), &mut signing_key_hex);
    let mut verifying_key_hex = [0; 64];
    hex(verifying_key.as_bytes(), &mut verifying_key_hex);
    println!(
        "\n\n--- Create a file named {config_file_name} with only the lines relevant to your \
         configuration ---\n\n"
    );
    println!("# Configuration for a client\n");
    println!("connect    = \"{}\"\t# edit appropriately", crate::DEFAULT_CONNECT);
    println!("psk        = \"{}\"", from_utf8(&psk_hex));
    println!("sign_pk    = \"{}\"", from_utf8(&verifying_key_hex));
    println!("sign_sk    = \"{}\"", from_utf8(&signing_key_hex));
    println!("encrypt_sk = \"{}\"", from_utf8(&encrypt_sk_hex));
    println!();
    println!("# Configuration for a server\n");
    println!("listen     = \"{}\"\t# edit appropriately", crate::DEFAULT_LISTEN);
    println!("psk        = \"{}\"", from_utf8(&psk_hex));
    println!("sign_pk    = \"{}\"", from_utf8(&verifying_key_hex));
    println!();
    println!("# Hybrid configuration\n");
    println!("connect    = \"{}\"\t# edit appropriately", crate::DEFAULT_CONNECT);
    println!("listen     = \"{}\"\t# edit appropriately", crate::DEFAULT_LISTEN);
    println!("psk        = \"{}\"", from_utf8(&psk_hex));
    println!("sign_pk    = \"{}\"", from_utf8(&verifying_key_hex));
    println!("sign_sk    = \"{}\"", from_utf8(&signing_key_hex));
    println!("encrypt_sk = \"{}\"", from_utf8(&encrypt_sk_hex));
}

#[inline]
#[allow(unsafe_code)]
const fn from_utf8(b: &[u8]) -> &str {
    unsafe { std::str::from_utf8_unchecked(b) }
}
