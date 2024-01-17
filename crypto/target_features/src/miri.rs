//! Minimal miri support.
//!
//! miri is an interpreter, and though it tries to emulate the target CPU, it
//! doesn't support target features

#[macro_export]
#[doc(hidden)]
macro_rules! __unless {
    ($($tf:tt),+ => $body:expr) => {
        false
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __detect {
    ($($tf:tt),+) => {
        false
    };
}
