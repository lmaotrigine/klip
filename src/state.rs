use crate::{error::Error, server::handle_connection, util::Stream};
use parking_lot::{Mutex, RwLock};
use std::{
    collections::{HashMap, VecDeque},
    net::IpAddr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::{net::TcpStream, sync::mpsc};

#[derive(PartialEq, Eq)]
pub enum Message {
    Chunk(Vec<u8>),
    End,
}

pub struct StreamHub {
    pub push_active: bool,
    pub pullers: HashMap<u64, mpsc::Sender<Message>>,
    pub next_id: u64,
}

impl StreamHub {
    fn new() -> Self {
        Self { push_active: false, pullers: HashMap::new(), next_id: 0 }
    }
}

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
    pub(crate) psk: [u8; 32],
    pub(crate) sign_pk: ed25519_dalek::VerifyingKey,
    pub(crate) timeout: core::time::Duration,
    pub(crate) data_timeout: core::time::Duration,
    pub(crate) ttl: core::time::Duration,
    pub(crate) max_len: u64,
    pub(crate) max_stream_bytes: u64,
    pub(crate) max_stream_duration: core::time::Duration,
    pub(crate) max_waiting_pullers: usize,
    max_clients: usize,
    trusted_ip_count: usize,
    trusted_clients: RwLock<VecDeque<IpAddr>>,
    client_count: AtomicUsize,
    stream_hub: Mutex<StreamHub>,
}

impl State {
    pub fn new(server: &crate::Server) -> Self {
        let trusted_ip_count = match server.max_clients / 10 {
            0 => 1,
            n => n,
        };
        Self {
            psk: server.psk,
            sign_pk: server.sign_pk,
            timeout: server.timeout,
            data_timeout: server.data_timeout,
            ttl: server.ttl,
            max_clients: server.max_clients,
            max_len: server.max_len,
            max_stream_bytes: server.max_stream_bytes,
            max_stream_duration: server.max_stream_duration,
            max_waiting_pullers: server.max_waiting_pullers,
            trusted_ip_count,
            trusted_clients: RwLock::new(VecDeque::with_capacity(trusted_ip_count)),
            client_count: AtomicUsize::new(0),
            stream_hub: Mutex::new(StreamHub::new()),
        }
    }

    pub fn add_trusted_ip(&self, ip: IpAddr) {
        let mut lock = self.trusted_clients.write();
        if lock.len() >= self.trusted_ip_count {
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
                log::warn!("while handling connection: {e:?}");
            }
        });
    }

    pub fn maybe_accept_client(self: Arc<Self>, conn: TcpStream) -> Result<(), Error> {
        let remote_ip = conn.peer_addr()?.ip();
        let mut count;
        loop {
            count = self.client_count.load(Ordering::SeqCst);
            if count >= self.max_clients - self.trusted_ip_count && !self.is_trusted_ip(remote_ip) {
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
        conn.set_timeout(self.timeout);
        self.accept_client(conn);
        Ok(())
    }

    pub fn stream_hub(&self) -> parking_lot::MutexGuard<'_, StreamHub> {
        self.stream_hub.lock()
    }

    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "illumos",
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
