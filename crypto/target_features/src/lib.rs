#![no_std]
#![deny(
    dead_code,
    deprecated,
    future_incompatible,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::nursery
)]

//! This crate provides a macro to detect CPU features at runtime. It's intended
//! to use as a a temporary solution until [RFC 2725] adding first-class target
//! feature detection macros to [`core`] is implemented.
//!
//! # Supported target architectures
//! <div class="warning">
//!     Target features with an asterisk are unstable (nightly-only) and subject
//!     to change to match upstream name changes in the Rust standard library.
//! </div>
//!
//! ## `aarch64`
//!
//! Linux, iOS, and macOS/ARM only (ARM64 doesn't support OS-independent feature
//! detection)
//!
//! Target features:
//!
//! * `sha2`*
//! * `sha3`*
//!
//! ## `x86`/`x86_64`
//!
//! OS-independent and `#![no_std]` friendly
//!
//! Target features
//!
//! * `avx2`
//! * `avx512ifma`*
//! * `avx512vl`*
//! * `sha`
//! * `sse2`
//! * `sse3`
//! * `sse4.1`
//! * `ssse3`
//!
//! This only contains features that I need for optimizations inside this
//! monorepo. This isn't going to be generally useful unless you also need to
//! check for only these features.
//!
//! Please don't open issues to add features unless you're also going to be
//! providing the implementation.
//!
//! # Example
//!
//! ```rust
//! # #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
//! # {
//! // this creates a `cpuid_avx2` module
//! target_features::detect!(cpuid_avx2, "avx2");
//!
//! // `token` is a ZST value, which guarantees that the underlying value is
//! // properly instantiated, and allows us to omit the initialization step
//! let token: cpuid_avx2::Features = cpuid_avx2::init();
//!
//! if token.get() {
//!     println!("CPU supports AVX2 extensions");
//! } else {
//!     println!("AVX2 extensions are not supported");
//! }
//!
//! // if you only need to check the feature once, you can use the module's
//! // `get` function directly
//! let val = cpuid_avx2::get();
//! assert_eq!(val, token.get());
//! # }
//! ```
//!
//! Note that if target features are enabled via compiler options (e.g. using
//! `RUSTFLAGS`), the feature detection code will be omitted and the `get`
//! method/function will return `true`. This behaviour allows the compiler to
//! eliminate fallback code.
//!
//! After the first call, the result is cached in an
//! [`AtomicU8`][`core::sync::atomic::AtomicU8`], thus runtime overhead is
//! minimal.
//!
//! [RFC 2725]: https://github.com/rust-lang/rfcs/pull/2725

#[cfg(not(miri))]
#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(miri)]
mod miri;
#[cfg(not(miri))]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")))]
compile_error!("this thing only works on `aarch64`, `x86` and `x86_64` targets.");

/// Create a module containing the CPU feature detection code.
#[macro_export]
macro_rules! detect {
    ($name:ident, $($tf:tt),+$(,)?) => {
        #[allow(unused, clippy::missing_const_for_fn, clippy::inline_always)]
        mod $name {
            use core::sync::atomic::{AtomicU8, Ordering::Relaxed};
            const UNINIT: u8 = 255;
            static FEATURES: AtomicU8 = AtomicU8::new(UNINIT);
            #[derive(Debug, Clone, Copy)]
            pub struct Features(());
            impl Features {
                /// get the initialized value
                #[inline(always)]
                pub fn get(&self) -> bool {
                    $crate::__unless! {
                        $($tf),+ => {
                            FEATURES.load(Relaxed) == 1
                        }
                    }
                }
            }
            /// initialize the underlying value if needed and return it
            #[inline]
            pub fn get() -> bool {
                $crate::__unless! {
                    $($tf),+ => {
                        let val = FEATURES.load(Relaxed);
                        if val == UNINIT {
                            let res = $crate::__detect!($($tf),+);
                            FEATURES.store(u8::from(res), Relaxed);
                            res
                        } else {
                            val == 1
                        }
                    }
                }
            }
            /// initialize the underlying value if needed and return a ZST that
            /// can skip this step on subsequent calls to [`get`][Features::get]
            #[inline]
            pub fn init() -> Features {
                let _ = get();
                Features(())
            }
        }
    };
}
