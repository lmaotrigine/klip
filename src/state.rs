use crate::{config::Config, error::Error, server::handle_connection, util::Stream};
use parking_lot::RwLock;
use std::{
    collections::VecDeque,
    net::IpAddr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::net::TcpStream;

// i gave up on borrow checker appeasement and made these global, sue me.
#[derive(Clone)]
pub struct Content {
    pub ts: u64,
    pub signature: [u8; 64],
    pub ciphertext_with_encrypt_sk_and_nonce: Vec<u8>,
}

pub struct Storage {
    pub generation: u64,
    pub content: Option<Content>,
}

pub static STORAGE: RwLock<Storage> = RwLock::new(Storage { generation: 0, content: None });

pub struct State {
    config: Config,
    trusted_clients: RwLock<VecDeque<IpAddr>>,
    client_count: AtomicUsize,
}

impl State {
    pub fn new(config: Config) -> Self {
        let cap = config.trusted_ip_count();
        Self {
            config,
            trusted_clients: RwLock::new(VecDeque::with_capacity(cap)),
            client_count: AtomicUsize::new(0),
        }
    }

    pub const fn config(&self) -> &Config {
        &self.config
    }

    pub fn add_trusted_ip(&self, ip: IpAddr) {
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

    pub fn accept_client(self: Arc<Self>, mut conn: Stream) {
        tokio::spawn(async move {
            let res = handle_connection(&self, &mut conn).await;
            self.client_count.fetch_sub(1, Ordering::SeqCst);
            if let Err(e) = res {
                let _ = conn.shutdown().await;
                eprintln!("error: {e}");
            }
        });
    }

    pub fn maybe_accept_client(self: Arc<Self>, conn: TcpStream) -> Result<(), Error> {
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
                .compare_exchange_weak(count, count + 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                break;
            }
        }
        let mut conn = Stream::new(conn);
        conn.set_timeout(self.config().timeout());
        self.accept_client(conn);
        Ok(())
    }

    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "illumos"
    ))]
    pub async fn handle_siginfo() -> std::io::Result<()> {
        use std::{
            borrow::Cow,
            sync::LazyLock,
            time::{Duration, SystemTime, UNIX_EPOCH},
        };
        use tokio::signal::unix::{SignalKind, signal};
        static BIN: LazyLock<String> = LazyLock::new(|| {
            std::env::args_os()
                .next()
                .map_or_else(|| "klip".to_owned(), |s| s.to_string_lossy().into_owned())
        });
        let mut signal = signal(SignalKind::info())?;
        while signal.recv().await == Some(()) {
            let name = &*BIN;
            let ts = STORAGE.read().content.as_ref().map(|c| c.ts);
            if let Some(ts) = ts {
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
            } else {
                println!("{name}: the clipboard is empty");
            }
        }
        Ok(())
    }
}
