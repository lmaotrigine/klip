#![no_std]
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

pub mod blocks;
pub mod constant_time;
pub mod erase;
