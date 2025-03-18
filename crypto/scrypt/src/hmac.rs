use crypto_common::blocks::{Block, Buffer};
use sha256::Sha256;

struct Core {
    digest: Sha256,
    opad_digest: Sha256,
}

impl Clone for Core {
    fn clone(&self) -> Self {
        Self {
            digest: self.digest.clone(),
            opad_digest: self.opad_digest.clone(),
        }
    }
}

impl Core {
    #[inline(always)]
    fn new_from_slice(key: &[u8]) -> Self {
        let mut buf = get_der_key(key);
        for b in &mut buf {
            *b ^= 0x36;
        }
        let mut digest = Sha256::default();
        digest.update_blocks(core::slice::from_ref(&buf));
        for b in &mut buf {
            *b ^= 0x36 ^ 0x5c;
        }
        let mut opad_digest = Sha256::default();
        opad_digest.update_blocks(core::slice::from_ref(&buf));
        Self {
            digest,
            opad_digest,
        }
    }

    #[inline(always)]
    fn update_blocks(&mut self, blocks: &[Block<64>]) {
        self.digest.update_blocks(blocks);
    }

    #[inline(always)]
    fn finalize(&mut self, buffer: &mut Buffer<64>, out: &mut [u8; 32]) {
        let mut hash = [0; 32];
        self.digest.finalize(buffer, &mut hash);
        buffer.reset();
        let h = &mut self.opad_digest;
        buffer.digest_blocks(&hash, |b| h.update_blocks(b));
        h.finalize(buffer, out);
    }
}

#[derive(Clone)]
pub struct Hmac {
    core: Core,
    buffer: Buffer<64>,
}

impl core::fmt::Debug for Hmac {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Hmac { ... }")
    }
}

impl Hmac {
    #[inline]
    #[must_use]
    pub fn new_from_slice(key: &[u8]) -> Self {
        Self {
            core: Core::new_from_slice(key),
            buffer: Buffer::default(),
        }
    }

    #[inline]
    pub fn update(&mut self, input: &[u8]) {
        let Self { core, buffer } = self;
        buffer.digest_blocks(input, |blocks| core.update_blocks(blocks));
    }

    #[inline]
    #[must_use]
    pub fn finalize_fixed(mut self) -> [u8; 32] {
        let mut out = [0; 32];
        let Self { core, buffer } = &mut self;
        core.finalize(buffer, &mut out);
        out
    }
}

fn get_der_key(key: &[u8]) -> Block<64> {
    let mut der_key = [0; 64];
    if key.len() < 64 {
        der_key[..key.len()].copy_from_slice(key);
    } else {
        let hash = Sha256::digest(key);
        der_key[..32].copy_from_slice(&hash);
    }
    der_key
}
