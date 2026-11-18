mod __support {
    mod inner {
        pub trait Sealed {
            fn dbg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;
        }
    }
    impl inner::Sealed for [u8; 32] {
        fn dbg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            if f.alternate() {
                let mut buf = [0; 64];
                let s = crate::util::hex(self, &mut buf);
                core::fmt::Debug::fmt(s, f)
            } else {
                core::fmt::Debug::fmt(&"<hidden>", f)
            }
        }
    }

    impl inner::Sealed for [u8; 8] {
        fn dbg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            if f.alternate() {
                let mut buf = [0; 16];
                let s = crate::util::hex(self, &mut buf);
                core::fmt::Debug::fmt(s, f)
            } else {
                core::fmt::Debug::fmt(&"<hidden>", f)
            }
        }
    }

    impl inner::Sealed for () {
        fn dbg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            core::fmt::Debug::fmt(&"<not set>", f)
        }
    }

    pub trait State: inner::Sealed + Copy {}
    impl<T: inner::Sealed + Copy> State for T {}
    pub struct Debug<T: State>(pub T);
    impl<T: State> core::fmt::Debug for Debug<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            self.0.dbg(f)
        }
    }
}

pub use __support::Debug;
use __support::State;

mod client;
mod server;

pub use client::Client;
pub use server::Server;
