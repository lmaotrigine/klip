use std::sync::Arc;

use crate::{
    authentication::{auth0, auth1, auth2get, auth2store, auth3get, auth3store},
    error::Error,
    state::{State, TS},
    util::Stream,
};
use crypto_common::constant_time::ConstantTimeEq;
use rand_core::RngCore;
use tokio::net::TcpListener;

struct Connection<'a> {
    stream: &'a mut Stream,
    state: &'a State,
}

impl<'a> Connection<'a> {
    pub async fn get_operation(self, h1: &[u8], is_move: bool) -> Result<(), Error> {
        let mut rbuf = [0; 32];
        self.stream.read_exact(&mut rbuf).await?;
        let h2 = rbuf;
        let opcode = if is_move { b'M' } else { b'G' };
        let wh2 = auth2get(self.state.config().psk(), h1, opcode);
        if wh2.as_bytes().ct_eq(&h2).to_u8() != 1 {
            return Err(Error::Auth);
        }
        let (ts, signature, ciphertext_with_encrypt_sk_and_nonce) = if is_move {
            let mut content = self.state.content.write();
            let mut ts = TS.write();
            let ret = (
                *ts,
                content.signature,
                content.ciphertext_with_encrypt_sk_and_nonce.clone(),
            );
            *ts = 0;
            content.signature = [0; 64];
            content.ciphertext_with_encrypt_sk_and_nonce.drain(..);
            drop(content);
            drop(ts);
            ret
        } else {
            let content = self.state.content.read();
            let ts = { *TS.read() };
            let signature = content.signature;
            let ciphertext_with_encrypt_sk_and_nonce =
                content.ciphertext_with_encrypt_sk_and_nonce.clone();
            drop(content);
            (ts, signature, ciphertext_with_encrypt_sk_and_nonce)
        };
        let signature = if signature == [0; 64] {
            &[]
        } else {
            &signature[..]
        };
        self.stream.set_timeout(self.state.config().data_timeout());
        let h3 = auth3get(self.state.config().psk(), &h2, &ts.to_le_bytes(), signature);
        self.stream.write_all(h3.as_bytes()).await?;
        let ciphertext_with_encrypt_sk_and_nonce_len =
            ciphertext_with_encrypt_sk_and_nonce.len() as u64;
        self.stream
            .write_all(&ciphertext_with_encrypt_sk_and_nonce_len.to_le_bytes())
            .await?;
        if ts == 0 {
            self.stream.flush().await?;
            return Ok(());
        }
        self.stream.write_all(&ts.to_le_bytes()).await?;
        self.stream.write_all(signature).await?;
        self.stream
            .write_all(&ciphertext_with_encrypt_sk_and_nonce)
            .await?;
        self.stream.flush().await?;
        Ok(())
    }

    #[allow(clippy::cast_possible_truncation)]
    pub async fn store_operation(self, h1: &[u8]) -> Result<(), Error> {
        let mut rbuf = [0; 112];
        self.stream.read_exact(&mut rbuf).await?;
        let h2 = &rbuf[..32];
        let len_buf: [u8; 8] = [
            rbuf[32], rbuf[33], rbuf[34], rbuf[35], rbuf[36], rbuf[37], rbuf[38], rbuf[39],
        ];

        let ciphertext_with_encrypt_sk_and_nonce_len = u64::from_le_bytes(len_buf);
        if ciphertext_with_encrypt_sk_and_nonce_len < 32 {
            return Err(Error::ShortCiphertext(
                ciphertext_with_encrypt_sk_and_nonce_len,
            ));
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
        let wh2 = auth2store(
            self.state.config().psk(),
            h1,
            opcode,
            &ts.to_le_bytes(),
            &signature,
        );
        if wh2.as_bytes().ct_eq(h2).to_u8() != 1 {
            return Err(Error::Auth);
        }
        let mut ciphertext_with_encrypt_sk_and_nonce =
            vec![0; ciphertext_with_encrypt_sk_and_nonce_len as usize];
        self.stream.set_timeout(self.state.config().data_timeout());
        self.stream
            .read_exact(&mut ciphertext_with_encrypt_sk_and_nonce)
            .await?;
        self.state.config().sign_pk().verify_strict(
            &ciphertext_with_encrypt_sk_and_nonce[..],
            &ed25519::Signature::from_bytes(&signature)?,
        )?;
        let h3 = auth3store(self.state.config().psk(), h2);
        {
            let mut content = self.state.content.write();
            *TS.write() = ts;
            content.signature = signature;
            content.ciphertext_with_encrypt_sk_and_nonce = ciphertext_with_encrypt_sk_and_nonce;
        }
        self.stream.set_timeout(self.state.config().data_timeout());
        self.stream.write_all(h3.as_bytes()).await?;
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
    if client_version != 1 {
        return Err(Error::IncompatibleVersions {
            client: client_version,
            server: 1,
        });
    }
    let r = &rbuf[1..33];
    let h0 = &rbuf[33..65];
    let wh0 = auth0(config.psk(), client_version, r);
    if wh0.as_bytes().ct_eq(h0).to_u8() != 1 {
        return Err(Error::Auth);
    }
    let mut r2 = [0; 32];
    let mut rand = rand_core::OsRng;
    rand.fill_bytes(&mut r2);
    let h1 = auth1(config.psk(), client_version, h0, &r2);
    stream.write_all(&[client_version]).await?;
    stream.write_all(&r2).await?;
    stream.write_all(h1.as_bytes()).await?;
    stream.flush().await?;
    state.add_trusted_ip(remote_addr.ip());
    let conn = Connection { stream, state };
    let mut opcode = [0];
    let opcode = conn
        .stream
        .read_exact(&mut opcode)
        .await
        .map(|_| opcode[0])?;
    match opcode {
        b'G' => conn.get_operation(h1.as_bytes(), false).await,
        b'M' => conn.get_operation(h1.as_bytes(), true).await,
        b'S' => conn.store_operation(h1.as_bytes()).await,
        _ => Err(Error::UnknownOp),
    }
}

pub async fn serve(state: State) -> Result<(), Error> {
    let state = Arc::new(state);
    tokio::spawn(async move { State::handle_siginfo().await });
    let listener = TcpListener::bind(state.config().listen()).await?;
    loop {
        let (conn, _) = listener.accept().await?;
        if let Err(e) = state.clone().maybe_accept_client(conn).await {
            eprintln!("error: {e}");
        }
    }
}
