use crate::DOMAIN;
use blake2::{
    Blake2bMac,
    digest::{Mac, typenum::U32},
};

type Blake2b = Blake2bMac<U32>;

fn new_blake2b(psk: [u8; 32], salt: u8) -> Blake2b {
    Blake2b::new_with_salt_and_personal(Some(&psk), &[salt], DOMAIN.as_bytes())
        .expect("invalid params in blake2b")
}

pub fn auth0(psk: [u8; 32], client_version: u8, r: &[u8]) -> [u8; 32] {
    let mut hf = new_blake2b(psk, 0);
    hf.update(&[client_version]);
    hf.update(r);
    hf.finalize().into_bytes().into()
}

pub fn derive_stream_key(
    encrypt_sk: [u8; 32],
    ts: u64,
    encrypt_sk_id: u64,
    nonce_prefix: [u8; 16],
    cid_bytes: &[u8],
) -> [u8; 32] {
    let mut hf = Blake2b::new_with_salt_and_personal(Some(&encrypt_sk), &[], b"k-v2-stream-key")
        .expect("invalid params in blake2b");
    hf.update(&ts.to_le_bytes());
    hf.update(&encrypt_sk_id.to_le_bytes());
    hf.update(&nonce_prefix);
    let cid_len = u16::try_from(cid_bytes.len()).expect("cid shouldn't be larger than 64KiB");
    hf.update(&cid_len.to_le_bytes());
    hf.update(cid_bytes);
    hf.finalize().into_bytes().into()
}

pub fn compute_cid_bind(encrypt_sk: [u8; 32], cid_bytes: &[u8]) -> [u8; 32] {
    if cid_bytes.is_empty() {
        return [0; 32];
    }
    let mut hf = Blake2b::new_with_salt_and_personal(Some(&encrypt_sk), &[], b"k-v2-cid-bind")
        .expect("invalid params in blake2b");
    let cid_len = u16::try_from(cid_bytes.len()).expect("cid shouldn't be larger than 64KiB");
    hf.update(&cid_len.to_le_bytes());
    hf.update(cid_bytes);
    hf.finalize().into_bytes().into()
}

pub fn new_transcript_hash() -> Blake2b {
    Blake2b::new_with_salt_and_personal(None, &[], b"k-v2-transcript")
        .expect("invalid params in blake2b")
}

pub fn derive_chunk_nonce(nonce_prefix: [u8; 16], chunk_index: u64) -> [u8; 24] {
    let mut nonce = [0; 24];
    nonce[..16].copy_from_slice(&nonce_prefix);
    nonce[16..].copy_from_slice(&chunk_index.to_le_bytes());
    nonce
}

pub fn auth1(psk: [u8; 32], client_version: u8, h0: &[u8], r2: &[u8]) -> [u8; 32] {
    let mut hf = new_blake2b(psk, 1);
    hf.update(&[client_version]);
    hf.update(r2);
    hf.update(h0);
    hf.finalize().into_bytes().into()
}

pub fn auth2get(psk: [u8; 32], h1: &[u8], opcode: u8) -> [u8; 32] {
    let mut hf = new_blake2b(psk, 2);
    hf.update(h1);
    hf.update(&[opcode]);
    hf.finalize().into_bytes().into()
}

pub fn auth2store(psk: [u8; 32], h1: &[u8], opcode: u8, ts: &[u8], signature: &[u8]) -> [u8; 32] {
    let mut hf = new_blake2b(psk, 2);
    hf.update(h1);
    hf.update(&[opcode]);
    hf.update(ts);
    hf.update(signature);
    hf.finalize().into_bytes().into()
}

pub fn auth3get(psk: [u8; 32], h2: &[u8], ts: &[u8], signature: &[u8]) -> [u8; 32] {
    let mut hf = new_blake2b(psk, 3);
    hf.update(h2);
    hf.update(ts);
    hf.update(signature);
    hf.finalize().into_bytes().into()
}

pub fn auth3store(psk: [u8; 32], h2: &[u8]) -> [u8; 32] {
    let mut hf = new_blake2b(psk, 3);
    hf.update(h2);
    hf.finalize().into_bytes().into()
}
