use crate::DOMAIN;

pub fn auth0(psk: [u8; 32], client_version: u8, r: &[u8]) -> blake2b::Result {
    let mut hf = blake2b::Blake2b::new_with_params(&[0], DOMAIN.as_bytes(), &psk, 32);
    hf.update(&[client_version]);
    hf.update(r);
    hf.finalize()
}

pub fn auth1(psk: [u8; 32], client_version: u8, h0: &[u8], r2: &[u8]) -> blake2b::Result {
    let mut hf = blake2b::Blake2b::new_with_params(&[1], DOMAIN.as_bytes(), &psk, 32);
    hf.update(&[client_version]);
    hf.update(r2);
    hf.update(h0);
    hf.finalize()
}

pub fn auth2get(psk: [u8; 32], h1: &[u8], opcode: u8) -> blake2b::Result {
    let mut hf = blake2b::Blake2b::new_with_params(&[2], DOMAIN.as_bytes(), &psk, 32);
    hf.update(h1);
    hf.update(&[opcode]);
    hf.finalize()
}

pub fn auth2store(
    psk: [u8; 32],
    h1: &[u8],
    opcode: u8,
    ts: &[u8],
    signature: &[u8],
) -> blake2b::Result {
    let mut hf = blake2b::Blake2b::new_with_params(&[2], DOMAIN.as_bytes(), &psk, 32);
    hf.update(h1);
    hf.update(&[opcode]);
    hf.update(ts);
    hf.update(signature);
    hf.finalize()
}

pub fn auth3get(psk: [u8; 32], h2: &[u8], ts: &[u8], signature: &[u8]) -> blake2b::Result {
    let mut hf = blake2b::Blake2b::new_with_params(&[3], DOMAIN.as_bytes(), &psk, 32);
    hf.update(h2);
    hf.update(ts);
    hf.update(signature);
    hf.finalize()
}

pub fn auth3store(psk: [u8; 32], h2: &[u8]) -> blake2b::Result {
    let mut hf = blake2b::Blake2b::new_with_params(&[3], DOMAIN.as_bytes(), &psk, 32);
    hf.update(h2);
    hf.finalize()
}
