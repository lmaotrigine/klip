use crate::hmac::Hmac;

#[inline]
#[allow(clippy::cast_possible_truncation)]
pub fn pbkdf2(password: &[u8], salt: &[u8], rounds: u32, res: &mut [u8]) {
    let hmac = Hmac::new_from_slice(password);
    for (i, chunk) in res.chunks_mut(32).enumerate() {
        inner(i as u32, chunk, &hmac, salt, rounds);
    }
}

#[inline(always)]
fn inner(i: u32, chunk: &mut [u8], hmac: &Hmac, salt: &[u8], rounds: u32) {
    for v in chunk.iter_mut() {
        *v = 0;
    }
    let mut salt = {
        let mut hmac_clone = hmac.clone();
        hmac_clone.update(salt);
        hmac_clone.update(&(i + 1).to_be_bytes());
        let salt = hmac_clone.finalize_fixed();
        xor(chunk, &salt);
        salt
    };
    for _ in 1..rounds {
        let mut hmac_clone = hmac.clone();
        hmac_clone.update(&salt);
        salt = hmac_clone.finalize_fixed();
        xor(chunk, &salt);
    }
}

#[inline(always)]
fn xor(res: &mut [u8], salt: &[u8]) {
    debug_assert!(salt.len() >= res.len(), "length mismatch in xor");
    res.iter_mut().zip(salt.iter()).for_each(|(a, b)| *a ^= b);
}
