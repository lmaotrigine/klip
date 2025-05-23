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
    unsafe_code,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used
)]

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

const EXPANDED_VERSION: &str = concat!(
    "v",
    env!("CARGO_PKG_VERSION"),
    env!("KLIP_BUILD_GIT_HASH"),
    " (protocol version ",
    default_client_version!(),
    ")"
);

const DOMAIN: &str = "KLIP";
const DEFAULT_LISTEN: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8075);
const DEFAULT_CONNECT: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8075);
const DEFAULT_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

mod authentication;
mod cli;
mod client;
mod config;
mod error;
mod keygen;
mod server;
mod state;
mod util;

use cli::Cli;

// since Rust no longer uses jemalloc by default, klip will, by default, use the
// system allocator. on linux, this would normally be glibc's allocator, which
// is decent. in particular, klip does not have a particularly allocation-heavy
// workload, so there isn't much difference (for our purposes) between glibc's
// allocator and jemalloc.
//
// however, when klip is built with musl, this means klip will use musl's
// allocator, which appears to be substantially worse. musl's goal is not to
// have the fastest version of everything; its goal is to be small and amenable
// to static compilation. even though klip isn't particularly allocation heavy,
// musl's allocator appears to slow down klip quite a bit. therefore, when
// building with musl, we use jemalloc.
//
// we don't unconditionally use jemalloc because it can be nice to use the
// system's default allocator by default. moreover, jemalloc seems to increase
// compilation times by a bit.
//
// we also only do this on 64-bit systems because jemalloc doesn't support i686.
#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[allow(clippy::redundant_pub_crate)] // macro generated
async fn shutdown() {
    let ctrlc = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install ^C handler");
    };
    #[cfg(unix)]
    let term = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let term = core::future::pending::<()>();
    tokio::select! {
        () = ctrlc => (),
        () = term => (),
    }
}

#[allow(clippy::needless_return, clippy::redundant_pub_crate)] // macro generated
#[tokio::main]
async fn main() -> Result<(), error::Context> {
    #[cfg(windows)]
    platform::preflight();
    tokio::select! {
        r = async { Cli::run().await } => r,
        () = shutdown() => {
            eprintln!("violently shutting down");
            std::process::exit(1);
        },
    }
}
