#![warn(missing_docs)]
#![cfg_attr(all(doc, not(doctest)), feature(doc_cfg))]

//! # klip
//!
//! A protocol for copying and pasting data over the network. It works by
//! sending content to an intermediary server, which can then be retrieved by
//! any client with the correct credentials. This allows for sharing data across
//! hosts that are behind different NATs, firewalls, or are otherwise unable to
//! communicate directly.
//!
//! The server does not know much about the data it is handling, and can
//! therefore be exposed to the public internet without much risk.

use core::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

const DOMAIN: &str = "KLIP";
const DEFAULT_LISTEN: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8075);
const DEFAULT_CONNECT: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8075);
const DEFAULT_TTL: Duration = Duration::from_hours(7 * 24);
const MAX_FUTURE_SKEW: Duration = Duration::from_hours(1);
const MAX_CHUNK_SIZE: u32 = 65536;
const DEFAULT_MAX_WAITING_PULLERS: usize = 100;
const DEFAULT_MAX_STREAM_BYTES: u64 = 10 << 30; // 10 GiB
const DEFAULT_MAX_STREAM_DURATION: Duration = Duration::from_hours(24);

mod authentication;
mod builder;
#[cfg(feature = "cli")]
mod cli;
mod client;
mod error;
mod keygen;
mod server;
mod state;
mod util;

pub use builder::{Client as ClientBuilder, Server as ServerBuilder};
#[cfg(any(feature = "cli", doc))]
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "cli")))]
pub use cli::Cli;
pub use client::Client;
pub use error::Error;
pub use keygen::Keys;
pub use server::Server;
pub use util::{Reader, Writer};
