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
