#![deny(
    dead_code,
    deprecated,
    future_incompatible,
    missing_copy_implementations,
    missing_debug_implementations,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::nursery
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::inline_always
)]

mod hmac;
mod params;
mod pbkdf2;
mod romix;
mod salsa;
pub use params::Params;
use pbkdf2::pbkdf2;

pub fn scrypt(
    password: &[u8],
    salt: &[u8],
    params: &Params,
    output: &mut [u8],
) -> Result<(), &'static str> {
    if output.is_empty() || output.len() / 32 > 0xffff_ffff {
        return Err("invalid length");
    }
    let n = 1 << params.log_n;
    let r128 = (params.r as usize) * 128;
    let p_r128 = (params.p as usize) * r128;
    let n_r128 = n * r128;
    let mut b = vec![0; p_r128];
    pbkdf2(password, salt, 1, &mut b);
    let mut v = vec![0; n_r128];
    let mut t = vec![0; r128];
    for chunk in &mut b.chunks_mut(r128) {
        romix::scrypt_ro_mix(chunk, &mut v, &mut t, n);
    }
    pbkdf2(password, &b, 1, output);
    Ok(())
}
