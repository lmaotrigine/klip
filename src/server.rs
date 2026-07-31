use crate::{
    authentication::{auth0, auth1, auth2get, auth2store, auth3get, auth3store},
    client::DEFAULT_CLIENT_VERSION,
    error::Error,
    state::{CONTENT, Content, State},
    util::Stream,
};
use ctutils::CtEq;
use rand::Rng;
use std::sync::Arc;
use tokio::net::TcpListener;

struct Connection<'a> {
    stream: &'a mut Stream,
    state: &'a State,
}

impl Connection<'_> {
    pub async fn get_operation(self, h1: &[u8], is_move: bool) -> Result<(), Error> {
        let mut rbuf = [0; 32];
        self.stream.read_exact(&mut rbuf).await?;
        let h2 = rbuf;
        let opcode = if is_move { b'M' } else { b'G' };
        let wh2 = auth2get(self.state.config().psk(), h1, opcode);
        let choice = wh2.ct_ne(&h2);
        if choice.into() {
            return Err(Error::Auth);
        }
        let (content, guard) = if is_move {
            let guard = CONTENT.write().await;
            (guard.clone(), Some(guard))
        } else {
            let content = CONTENT.read().await.clone();
            (content, None)
        };
        let Some(Content { ts, signature, ciphertext_with_encrypt_sk_and_nonce }) = content else {
            self.stream.write_all(&[0]).await?;
            self.stream.flush().await?;
            return Ok(());
        };
        self.stream.set_timeout(self.state.config().data_timeout());
        let h3 = auth3get(self.state.config().psk(), &h2, &ts.to_le_bytes(), &signature);
        self.stream.write_all(&h3).await?;
        let ciphertext_with_encrypt_sk_and_nonce_len =
            ciphertext_with_encrypt_sk_and_nonce.len() as u64;
        self.stream.write_all(&ciphertext_with_encrypt_sk_and_nonce_len.to_le_bytes()).await?;
        if ts == 0 {
            self.stream.flush().await?;
            return Ok(());
        }
        self.stream.write_all(&ts.to_le_bytes()).await?;
        self.stream.write_all(&signature).await?;
        self.stream.write_all(&ciphertext_with_encrypt_sk_and_nonce).await?;
        self.stream.flush().await?;
        if let Some(mut guard) = guard {
            *guard = None;
        }
        Ok(())
    }

    pub async fn store_operation(self, h1: &[u8]) -> Result<(), Error> {
        let mut rbuf = [0; 112];
        self.stream.read_exact(&mut rbuf).await?;
        let h2 = &rbuf[..32];
        let mut len_buf = [0; 8];
        len_buf.copy_from_slice(&rbuf[32..40]);
        let ciphertext_with_encrypt_sk_and_nonce_len = u64::from_le_bytes(len_buf);
        if ciphertext_with_encrypt_sk_and_nonce_len < 32 {
            return Err(Error::ShortCiphertext(ciphertext_with_encrypt_sk_and_nonce_len));
        }
        if self.state.config().max_len() > 0
            && ciphertext_with_encrypt_sk_and_nonce_len > self.state.config().max_len()
        {
            return Err(Error::Large {
                max: self.state.config().max_len(),
                got: ciphertext_with_encrypt_sk_and_nonce_len,
            });
        }
        let mut tsbuf = [0; 8];
        tsbuf.copy_from_slice(&rbuf[40..48]);
        let ts = u64::from_le_bytes(tsbuf);
        let mut signature = [0; 64];
        signature.copy_from_slice(&rbuf[48..112]);
        let opcode = b'S';
        let wh2 = auth2store(self.state.config().psk(), h1, opcode, &ts.to_le_bytes(), &signature);
        let choice = wh2.as_slice().ct_ne(h2);
        if choice.into() {
            return Err(Error::Auth);
        }
        let full_buf_len = ciphertext_with_encrypt_sk_and_nonce_len.try_into().map_err(|_| {
            Error::Large { max: usize::MAX as _, got: ciphertext_with_encrypt_sk_and_nonce_len }
        })?;
        let mut ciphertext_with_encrypt_sk_and_nonce = vec![0; full_buf_len];
        self.stream.set_timeout(self.state.config().data_timeout());
        self.stream.read_exact(&mut ciphertext_with_encrypt_sk_and_nonce).await?;
        self.state.config().sign_pk().verify_strict(
            &ciphertext_with_encrypt_sk_and_nonce,
            &ed25519_dalek::Signature::from_bytes(&signature),
        )?;
        let h3 = auth3store(self.state.config().psk(), h2);
        {
            let content = Content { ts, signature, ciphertext_with_encrypt_sk_and_nonce };
            *CONTENT.write().await = Some(content);
        }
        self.stream.set_timeout(self.state.config().data_timeout());
        self.stream.write_all(&h3).await?;
        self.stream.flush().await?;
        Ok(())
    }
}

pub async fn handle_connection(state: &State, stream: &mut Stream) -> Result<(), Error> {
    let config = state.config();
    let mut rbuf = [0; 65];
    let remote_addr = stream.peer_addr()?;
    stream.read_exact(&mut rbuf).await?;
    let client_version = rbuf[0];
    if client_version != DEFAULT_CLIENT_VERSION {
        return Err(Error::IncompatibleVersions {
            client: client_version,
            server: DEFAULT_CLIENT_VERSION,
        });
    }
    let r = &rbuf[1..33];
    let h0 = &rbuf[33..65];
    let wh0 = &auth0(config.psk(), client_version, r)[..];
    let choice = wh0.ct_ne(h0);
    if choice.into() {
        return Err(Error::Auth);
    }
    let mut r2 = [0; 32];
    let mut rand = rand::make_rng::<rand::rngs::StdRng>();
    rand.fill_bytes(&mut r2);
    let h1 = auth1(config.psk(), client_version, h0, &r2);
    stream.write_all(&[client_version]).await?;
    stream.write_all(&r2).await?;
    stream.write_all(&h1).await?;
    stream.flush().await?;
    state.add_trusted_ip(remote_addr.ip());
    let conn = Connection { stream, state };
    let mut opcode = [0];
    let opcode = conn.stream.read_exact(&mut opcode).await.map(|_| opcode[0])?;
    match opcode {
        b'G' => conn.get_operation(&h1, false).await,
        b'M' => conn.get_operation(&h1, true).await,
        b'S' => conn.store_operation(&h1).await,
        _ => Err(Error::UnknownOp),
    }
}

pub async fn serve(state: State) -> Result<(), Error> {
    let state = Arc::new(state);
    #[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    tokio::spawn(async move { State::handle_siginfo().await });
    let listener = TcpListener::bind(state.config().listen()).await?;
    loop {
        let (conn, _) = listener.accept().await?;
        if let Err(e) = state.clone().maybe_accept_client(conn) {
            eprintln!("error: {e}");
        }
    }
}
