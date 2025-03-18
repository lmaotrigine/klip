#![no_std]
#![forbid(unsafe_code)]
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

mod consts;
mod errors;
mod hazmat;
mod sign;
mod signature;
mod verify;

pub use errors::SignatureError;
pub use sign::SigningKey;
pub use signature::Signature;
pub use verify::VerifyingKey;
