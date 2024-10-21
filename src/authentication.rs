use crate::DOMAIN;

fn new_blake2b(psk: [u8; 32], salt: u8) -> blake2b::State {
    let mut p = blake2b::Params::new();
    p.hash_length(32)
        .salt(&[salt])
        .personal(DOMAIN.as_bytes())
        .key(&psk);
    p.to_state()
}

pub fn auth0(psk: [u8; 32], client_version: u8, r: &[u8]) -> blake2b::Hash {
    let mut hf = new_blake2b(psk, 0);
    hf.update(&[client_version]);
    hf.update(r);
    hf.finalize()
}

pub fn auth1(psk: [u8; 32], client_version: u8, h0: &[u8], r2: &[u8]) -> blake2b::Hash {
    let mut hf = new_blake2b(psk, 1);
    hf.update(&[client_version]);
    hf.update(r2);
    hf.update(h0);
    hf.finalize()
}

pub fn auth2get(psk: [u8; 32], h1: &[u8], opcode: u8) -> blake2b::Hash {
    let mut hf = new_blake2b(psk, 2);
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
) -> blake2b::Hash {
    let mut hf = new_blake2b(psk, 2);
    hf.update(h1);
    hf.update(&[opcode]);
    hf.update(ts);
    hf.update(signature);
    hf.finalize()
}

pub fn auth3get(psk: [u8; 32], h2: &[u8], ts: &[u8], signature: &[u8]) -> blake2b::Hash {
    let mut hf = new_blake2b(psk, 3);
    hf.update(h2);
    hf.update(ts);
    hf.update(signature);
    hf.finalize()
}

pub fn auth3store(psk: [u8; 32], h2: &[u8]) -> blake2b::Hash {
    let mut hf = new_blake2b(psk, 3);
    hf.update(h2);
    hf.finalize()
}
