use crate::{config::Config, error::Error, server::handle_connection, util::Stream};
use parking_lot::RwLock;
use std::{
    collections::VecDeque,
    net::IpAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::net::TcpStream;

// i gave up on borrow checker appeasement and made these global, sue me.
pub static TS: RwLock<u64> = RwLock::new(0);
#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
static ARGV0: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub struct Content {
    pub signature: [u8; 64],
    pub ciphertext_with_encrypt_sk_and_nonce: Vec<u8>,
}

pub struct State {
    config: Config,
    trusted_clients: RwLock<VecDeque<IpAddr>>,
    client_count: AtomicUsize,
    pub content: Arc<RwLock<Content>>,
}

impl State {
    pub fn new(config: Config) -> Self {
        let cap = config.trusted_ip_count();
        Self {
            config,
            trusted_clients: RwLock::new(VecDeque::with_capacity(cap)),
            client_count: AtomicUsize::new(0),
            content: Arc::new(RwLock::new(Content {
                signature: [0; 64],
                ciphertext_with_encrypt_sk_and_nonce: Vec::new(),
            })),
        }
    }

    pub const fn config(&self) -> &Config {
        &self.config
    }

    pub fn add_trusted_ip(&mut self, ip: IpAddr) {
        let mut lock = self.trusted_clients.write();
        if lock.len() >= self.config.trusted_ip_count() {
            lock.pop_front();
        }
        lock.push_back(ip);
    }

    pub fn is_trusted_ip(&self, ip: IpAddr) -> bool {
        let g = self.trusted_clients.read();
        g.is_empty() || g.contains(&ip)
    }

    pub async fn accept_client(&mut self, mut conn: Stream) -> Result<(), Error> {
        if let Err(e) = handle_connection(self, &mut conn).await {
            self.client_count.fetch_sub(1, Ordering::SeqCst);
            conn.shutdown().await?;
            return Err(e);
        }
        self.client_count.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }

    pub async fn maybe_accept_client(&mut self, conn: TcpStream) -> Result<(), Error> {
        let remote_ip = conn.peer_addr()?.ip();
        let mut count;
        loop {
            count = self.client_count.load(Ordering::SeqCst);
            if count >= self.config.max_clients() - self.config.trusted_ip_count()
                && !self.is_trusted_ip(remote_ip)
            {
                return Err(Error::CapacityReached);
            }
            if self
                .client_count
                .compare_exchange(count, count + 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                break;
            }
        }
        let mut conn = Stream::new(conn);
        conn.set_timeout(self.config().timeout());
        self.accept_client(conn).await
    }

    #[cfg_attr(
        not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd"
        )),
        allow(clippy::unnecessary_wraps, clippy::unused_async)
    )]
    pub async fn handle_siginfo() -> std::io::Result<()> {
        #[cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            use std::{
                borrow::Cow,
                time::{Duration, SystemTime, UNIX_EPOCH},
            };
            use tokio::signal::unix::{signal, SignalKind};
            let mut signal = signal(SignalKind::info())?;
            while signal.recv().await == Some(()) {
                let name = ARGV0
                    .get_or_init(|| std::env::args().next().unwrap_or_else(|| "klip".to_owned()));
                let value = *TS.read();
                match value {
                    0 => println!("{name}: the clipboard is empty"),
                    ts => {
                        let elapsed = SystemTime::now()
                            .duration_since(UNIX_EPOCH + Duration::from_secs(ts))
                            .unwrap_or_default()
                            .as_secs()
                            / 60;
                        let msg = if elapsed <= 1 {
                            Cow::Borrowed("a few moments ago")
                        } else {
                            Cow::Owned(format!("{elapsed} minutes ago"))
                        };
                        println!("{name}: the clipboard is not empty (last filled {msg})");
                    }
                }
            }
            Ok(())
        }
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        {
            Ok(())
        }
    }
}
