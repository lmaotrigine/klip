mod __support {
    pub trait State: Copy {
        // technically this would be more readable in a separate trait, but
        // since all of it would be private anyway, just do this here.
        // basically, this hides sensitive info unless {:#?} is used, in which
        // case it is displayed as hex.
        fn dbg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;
    }

    impl State for () {
        fn dbg(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            core::fmt::Debug::fmt(&"<not set>", f)
        }
    }

    impl State for [u8; 32] {
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

    // this is not a state, but we do want it to have the dbg method
    // ugly? probably. but no one's looking at this code... right?
    impl State for [u8; 8] {
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
