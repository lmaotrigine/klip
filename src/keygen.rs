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

pub fn generate_keys(config_file_name: impl core::fmt::Display, key: &[u8]) {
    let mut rng = if key.is_empty() {
        Rand::OsRng(rand::rng())
    } else {
        Rand::Deterministic(DeterministicRandom::init(key))
    };
    let mut psk_buf = [0; 32];
    let mut psk_hex_buf = [0; 64];
    rng.fill_bytes(&mut psk_buf);
    let psk = hex(&psk_buf, &mut psk_hex_buf);
    let mut encrypt_sk_buf = [0; 32];
    let mut encrypt_sk_hex_buf = [0; 64];
    rng.fill_bytes(&mut encrypt_sk_buf);
    let encrypt_sk = hex(&encrypt_sk_buf, &mut encrypt_sk_hex_buf);
    let mut signing_key = [0; 32];
    rng.fill_bytes(&mut signing_key);
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&signing_key);
    let verifying_key = signing_key.verifying_key();
    let mut signing_key_hex = [0; 64];
    let signing_key = hex(signing_key.as_bytes(), &mut signing_key_hex);
    let mut verifying_key_hex = [0; 64];
    let verifying_key = hex(verifying_key.as_bytes(), &mut verifying_key_hex);
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
