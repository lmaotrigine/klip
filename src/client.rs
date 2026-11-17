use crate::{
    MAX_CHUNK_SIZE, MAX_FUTURE_SKEW,
    authentication::{
        auth0, auth1, auth2get, auth2store, auth3get, auth3store, compute_cid_bind,
        derive_chunk_nonce, derive_stream_key, new_transcript_hash,
    },
    error::Error,
    util::{Reader, Stream, Writer},
};
use blake2::digest::Mac;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20poly1305::{KeyInit, aead::Aead};
use ctutils::CtEq;
use ed25519_dalek::Signer;
use rand::Rng;
use std::{
    io,
    net::SocketAddr,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub const DEFAULT_CLIENT_VERSION: u8 = 2;

/// A client for the klip protocol.
pub struct Client {
    pub(crate) addr: SocketAddr,
    pub(crate) encrypt_sk: [u8; 32],
    pub(crate) encrypt_sk_id: u64,
    pub(crate) psk: [u8; 32],
    pub(crate) sign_pk: ed25519_dalek::VerifyingKey,
    pub(crate) sign_sk: ed25519_dalek::SigningKey,
    pub(crate) timeout: Duration,
    pub(crate) data_timeout: Duration,
    pub(crate) ttl: Duration,
    pub(crate) max_stream_bytes: u64,
    pub(crate) max_stream_duration: Duration,
}

impl Client {
    pub(crate) async fn connect(&self) -> Result<Stream, Error> {
        let fut = tokio::net::TcpStream::connect(self.addr);
        let conn = tokio::time::timeout(self.timeout, fut).await.map_err(io::Error::from)??;
        Ok(Stream::new(conn))
    }

    async fn handshake<'a>(
        &self,
        stream: &mut Stream,
        buf: &'a mut [u8; 32],
    ) -> Result<&'a [u8; 32], Error> {
        let psk = self.psk;
        let mut r = [0; 32];
        let mut rng = rand::make_rng::<rand::rngs::StdRng>();
        rng.fill_bytes(&mut r);
        let h0 = auth0(psk, DEFAULT_CLIENT_VERSION, &r);
        stream.write_all(&[DEFAULT_CLIENT_VERSION]).await?;
        stream.write_all(&r).await?;
        stream.write_all(&h0).await?;
        stream.flush().await?;
        let mut rbuf = [0; 65];
        stream.read_exact(&mut rbuf).await.map_err(|_| {
            if rbuf.iter().position(|&b| b == 0).is_some_and(|p| p < 2) {
                Error::Message(
                    "the server rejected the connection - check that it is running the same klip \
                     version or retry later",
                )
            } else {
                Error::ProtocolUnsupported
            }
        })?;
        if rbuf[0] != DEFAULT_CLIENT_VERSION {
            return Err(Error::IncompatibleVersions {
                client: DEFAULT_CLIENT_VERSION,
                server: rbuf[0],
            });
        }
        let r2 = &rbuf[1..33];
        let h1 = &rbuf[33..65];
        let wh1 = &auth1(psk, DEFAULT_CLIENT_VERSION, &h0, r2)[..];
        let choice = wh1.ct_ne(h1);
        if choice.into() {
            return Err(Error::Auth);
        }
        buf.copy_from_slice(h1);
        Ok(buf)
    }

    /// Perform the copy operation. Content is read asynchronously from `reader`
    /// and sent to the server.
    ///
    /// # Errors
    ///
    /// This method fails if there was an error communicating with the server,
    /// if the server rejected the connection, or if there was an error reading
    /// from the reader.
    ///
    /// # Panics
    ///
    /// This method will panic in the unlikely event that the system clock is
    /// broken and reports the current time as being earlier than the UNIX
    /// epoch.
    pub async fn copy<R: Reader>(&mut self, mut reader: R) -> Result<(), Error> {
        let mut stream = self.connect().await?;
        let mut h1_buf = [0; 32];
        let h1 = self.handshake(&mut stream, &mut h1_buf).await?;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is broken")
            .as_secs()
            .to_le_bytes();
        let mut content_with_encrypt_sk_id_and_nonce = vec![0; 32];
        content_with_encrypt_sk_id_and_nonce[..8]
            .copy_from_slice(&self.encrypt_sk_id.to_le_bytes());
        let mut rng = rand::make_rng::<rand::rngs::StdRng>();
        rng.fill_bytes(&mut content_with_encrypt_sk_id_and_nonce[8..32]);
        reader.read_to_end(&mut content_with_encrypt_sk_id_and_nonce).await?;
        let opcode = b'S';
        let mut cipher = chacha20::XChaCha20::new(
            &self.encrypt_sk.into(),
            &content_with_encrypt_sk_id_and_nonce[8..32]
                .try_into()
                .expect("8..32 doesn't span 24 bytes. math has died."),
        );
        let ct = &mut content_with_encrypt_sk_id_and_nonce[32..];
        cipher.apply_keystream(ct);
        let signature = self.sign_sk.sign(content_with_encrypt_sk_id_and_nonce.as_slice());
        stream.set_timeout(self.data_timeout);
        let h2 = auth2store(self.psk, h1, opcode, &ts, &signature.to_bytes());
        let h3 = async {
            stream.write_all(&[opcode]).await?;
            stream.write_all(&h2).await?;
            let ciphertext_with_encrypt_sk_id_and_nonce_len =
                content_with_encrypt_sk_id_and_nonce.len() as u64;
            stream.write_all(&ciphertext_with_encrypt_sk_id_and_nonce_len.to_le_bytes()).await?;
            stream.write_all(&ts).await?;
            stream.write_all(&signature.to_bytes()).await?;
            stream.write_all(&content_with_encrypt_sk_id_and_nonce).await?;
            stream.flush().await?;
            let mut h3 = [0; 32];
            stream.read_exact(&mut h3).await?;
            Ok::<_, io::Error>(h3)
        }
        .await
        .map_err(|e| {
            if matches!(e.kind(), io::ErrorKind::UnexpectedEof | io::ErrorKind::BrokenPipe) {
                Error::ConnectionClosed
            } else {
                e.into()
            }
        })?;
        let wh3 = auth3store(self.psk, &h2);
        let choice = wh3.ct_ne(&h3);
        if choice.into() {
            return Err(Error::Auth);
        }
        Ok(())
    }

    /// Perform the paste operation. Content is written asynchronously to
    /// `writer`.
    ///
    /// # Errors
    ///
    /// This method fails if there was an error communicating with the server,
    /// if the server rejected the connection, or if there was an error writing
    /// to the writer.
    pub async fn paste<W: Writer>(&mut self, mut writer: W, is_move: bool) -> Result<(), Error> {
        let mut stream = self.connect().await?;
        let opcode = if is_move { b'M' } else { b'G' };
        let mut h1_buf = [0; 32];
        let h1 = self.handshake(&mut stream, &mut h1_buf).await?;
        let h2 = auth2get(self.psk, h1, opcode);
        stream.write_all(&[opcode]).await?;
        stream.write_all(&h2).await?;
        stream.flush().await?;
        let mut rbuf = [0; 112];
        stream.read_exact(&mut rbuf).await.map_err(|e| {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                if rbuf.iter().position(|&b| b == 0).is_some_and(|p| p > 80) {
                    Error::MaybeIncompatibleVersion
                } else {
                    Error::Empty
                }
            } else {
                e.into()
            }
        })?;
        let mut len_buf = [0; 8];
        len_buf.copy_from_slice(&rbuf[32..40]);
        let h3 = &rbuf[..32];
        let ciphertext_with_encrypt_sk_id_and_nonce_len = u64::from_le_bytes(len_buf);
        let mut ts = [0; 8];
        ts.copy_from_slice(&rbuf[40..48]);
        let mut signature = [0; 64];
        signature.copy_from_slice(&rbuf[48..112]);
        let wh3 = &auth3get(self.psk, &h2, &ts, &signature)[..];
        let choice = wh3.ct_ne(h3);
        if choice.into() {
            return Err(Error::Auth);
        }
        let ts_val = UNIX_EPOCH + Duration::from_secs(u64::from_le_bytes(ts));
        let now = SystemTime::now();
        if ts_val > (now + MAX_FUTURE_SKEW) {
            return Err(Error::Future);
        }
        let elapsed = now.duration_since(ts_val).unwrap_or_else(|_| Duration::from_secs(0));
        if elapsed >= self.ttl {
            return Err(Error::Old);
        }
        if ciphertext_with_encrypt_sk_id_and_nonce_len < 32 {
            return Err(Error::Short(None));
        }
        let full_buf_len =
            ciphertext_with_encrypt_sk_id_and_nonce_len.try_into().map_err(|_| Error::Large {
                max: usize::MAX as _,
                got: ciphertext_with_encrypt_sk_id_and_nonce_len,
            })?;
        let mut ciphertext_with_encrypt_sk_id_and_nonce = vec![0; full_buf_len];
        stream.set_timeout(self.data_timeout);
        stream.read_exact(&mut ciphertext_with_encrypt_sk_id_and_nonce[..]).await.map_err(|e| {
            if matches!(e.kind(), io::ErrorKind::UnexpectedEof | io::ErrorKind::BrokenPipe) {
                Error::ConnectionClosed
            } else {
                e.into()
            }
        })?;
        let encrypt_sk_id = {
            let c = &ciphertext_with_encrypt_sk_id_and_nonce[..8];
            [c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]
        };
        let choice = encrypt_sk_id.ct_ne(&self.encrypt_sk_id.to_le_bytes());
        if choice.into() {
            let w_encrypt_sk = self.encrypt_sk_id;
            let encrypt_sk = u64::from_le_bytes(encrypt_sk_id);
            return Err(Error::SecretKeyIDMismatch { expected: w_encrypt_sk, actual: encrypt_sk });
        }
        self.sign_pk.verify_strict(
            &ciphertext_with_encrypt_sk_id_and_nonce[..],
            &ed25519_dalek::Signature::from_bytes(&signature),
        )?;
        let nonce = &ciphertext_with_encrypt_sk_id_and_nonce[8..32];
        let mut cipher = chacha20::XChaCha20::new(
            &self.encrypt_sk.into(),
            nonce
                .try_into()
                .unwrap_or_else(|_| unreachable!("8..32 doesn't span 24 bytes. math has died.")),
        );
        let content = &mut ciphertext_with_encrypt_sk_id_and_nonce[32..];
        cipher.apply_keystream(&mut content[..]);
        writer.write_all(content).await?;
        writer.flush().await?;
        Ok(())
    }

    /// Send content to waiting pullers in a streaming fashion.
    ///
    /// This is useful when sending large amounts of data over a potentially
    /// slow connection, or when the size of the data exceeds the memory
    /// capacity of the server.
    ///
    /// # Errors
    ///
    /// This method fails if there was an error communicating with the server,
    /// if the server rejected the connection, or if there was an error reading
    /// from the reader. The server will reject the connection if there are no
    /// waiting pullers, or if there is already an active push stream.
    ///
    /// # Panics
    ///
    /// This method will panic in the unlikely event that the system clock is
    /// broken and reports the current time as being earlier than the UNIX
    /// epoch.
    pub async fn push_stream<R: Reader>(&mut self, mut reader: R, cid: &str) -> Result<(), Error> {
        let mut stream = self.connect().await?;
        let opcode = b'P';
        let mut h1_buf = [0; 32];
        let h1 = self.handshake(&mut stream, &mut h1_buf).await?;
        let h2 = auth2get(self.psk, h1, opcode);
        stream.write_all(&[opcode]).await?;
        stream.write_all(&h2).await?;
        stream.flush().await?;
        stream.set_timeout(self.timeout);
        let mut status = [0];
        stream.read_exact(&mut status).await.map_err(|e| {
            if matches!(e.kind(), io::ErrorKind::UnexpectedEof | io::ErrorKind::BrokenPipe) {
                Error::ConnectionClosed
            } else {
                e.into()
            }
        })?;
        match status[0] {
            0x00 => return Err(Error::NoWaiters),
            0x01 => (),
            0x02 => return Err(Error::StreamActive { push: true }),
            _ => return Err(Error::GenericStream),
        }
        let cid_bytes = cid.as_bytes();
        let ts =
            SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock is broken").as_secs();
        let mut nonce_prefix = [0; 16];
        rand::make_rng::<rand::rngs::StdRng>().fill_bytes(&mut nonce_prefix);
        let stream_key =
            derive_stream_key(self.encrypt_sk, ts, self.encrypt_sk_id, nonce_prefix, cid_bytes);
        let aead = chacha20poly1305::XChaCha20Poly1305::new(&stream_key.into());
        let cid_bind = compute_cid_bind(self.encrypt_sk, cid_bytes);
        let mut transcript = new_transcript_hash();
        transcript.update(&[DEFAULT_CLIENT_VERSION, opcode]);
        transcript.update(&ts.to_le_bytes());
        transcript.update(&self.encrypt_sk_id.to_le_bytes());
        transcript.update(&nonce_prefix);
        transcript.update(&cid_bind);
        let mut header = [0; 32];
        header[..8].copy_from_slice(&ts.to_le_bytes());
        header[8..16].copy_from_slice(&self.encrypt_sk_id.to_le_bytes());
        header[16..32].copy_from_slice(&nonce_prefix);
        stream.set_timeout(self.data_timeout);
        stream.write_all(&header).await?;
        stream.flush().await?;
        let mut plain_buf = vec![0; MAX_CHUNK_SIZE as usize];
        let mut chunk_index = 0;
        loop {
            let res = reader.read(&mut plain_buf).await;
            match res {
                Ok(n) if n > 0 => {
                    let nonce = derive_chunk_nonce(nonce_prefix, chunk_index);
                    let sealed = aead.encrypt(&nonce.into(), &plain_buf[..n])?;
                    let chunk_len = u32::try_from(n).expect("chunk can't be too big for u32");
                    transcript.update(&chunk_index.to_le_bytes());
                    transcript.update(&chunk_len.to_le_bytes());
                    transcript.update(&sealed);
                    stream.set_timeout(self.data_timeout);
                    stream.write_all(&chunk_len.to_le_bytes()).await?;
                    stream.write_all(&sealed).await?;
                    stream.flush().await?;
                    chunk_index += 1;
                }
                // Ok(0)
                Ok(_) => break,
                Err(e) => {
                    if matches!(e.kind(), io::ErrorKind::UnexpectedEof | io::ErrorKind::BrokenPipe)
                    {
                        break;
                    }
                    return Err(e.into());
                }
            }
        }
        let transcript_digest = transcript.finalize().into_bytes();
        let signature = self.sign_sk.sign(&transcript_digest);
        let end_marker = [0; 4];
        stream.set_timeout(self.data_timeout);
        stream.write_all(&end_marker).await?;
        stream.write_all(&signature.to_bytes()).await?;
        stream.flush().await?;
        Ok(())
    }

    /// Wait for a stream to be sent by a push operation and write it
    /// asynchronously to `writer`.
    ///
    /// # Errors
    ///
    /// This method fails if there was an error communicating with the server,
    /// if the server rejected the connection, or if there was an error writing
    /// to the writer. The server will reject the connection if there is already
    /// an active push stream, or if there are too many waiting pullers.
    pub async fn pull_stream<W: Writer>(&mut self, mut writer: W, cid: &str) -> Result<(), Error> {
        let mut stream = self.connect().await?;
        let opcode = b'L';
        let mut h1_buf = [0; 32];
        let h1 = self.handshake(&mut stream, &mut h1_buf).await?;
        let h2 = auth2get(self.psk, h1, opcode);
        stream.write_all(&[opcode]).await?;
        stream.write_all(&h2).await?;
        stream.flush().await?;
        stream.set_timeout(self.timeout);
        let mut status = [0];
        stream.read_exact(&mut status).await.map_err(|e| {
            if matches!(e.kind(), io::ErrorKind::UnexpectedEof | io::ErrorKind::BrokenPipe) {
                Error::ConnectionClosed
            } else {
                e.into()
            }
        })?;
        match status[0] {
            0x00 => return Err(Error::StreamActive { push: false }),
            0x01 => (),
            0x02 => return Err(Error::TooManyPullers),
            _ => return Err(Error::GenericStream),
        }
        let cid_bytes = cid.as_bytes();
        stream.clear_timeout();
        let mut header = [0; 32];
        stream.read_exact(&mut header).await.inspect_err(|e| {
            log::error!("stream: failed to read header: {e:?}");
        })?;
        let ts = u64::from_le_bytes(
            header[..8].try_into().unwrap_or_else(|_| unreachable!("0..8 is 8 bytes lol")),
        );
        let encrypt_sk_id = u64::from_le_bytes(
            header[8..16].try_into().unwrap_or_else(|_| unreachable!("8..16 is 8 bytes lol")),
        );
        let nonce_prefix =
            header[16..32].try_into().unwrap_or_else(|_| unreachable!("16..32 is 16 bytes lol"));
        let now = SystemTime::now();
        let ts_val = UNIX_EPOCH + Duration::from_secs(ts);
        match now.duration_since(ts_val) {
            Ok(elapsed) if elapsed > self.ttl => {
                return Err(Error::Old);
            }
            Err(skew) if skew.duration() > MAX_FUTURE_SKEW => {
                return Err(Error::Future);
            }
            _ => (),
        }
        if encrypt_sk_id != self.encrypt_sk_id {
            return Err(Error::SecretKeyIDMismatch {
                expected: self.encrypt_sk_id,
                actual: encrypt_sk_id,
            });
        }
        let stream_key =
            derive_stream_key(self.encrypt_sk, ts, encrypt_sk_id, nonce_prefix, cid_bytes);
        let aead = chacha20poly1305::XChaCha20Poly1305::new(&stream_key.into());
        let cid_bind = compute_cid_bind(self.encrypt_sk, cid_bytes);
        let mut transcript = new_transcript_hash();
        transcript.update(&[DEFAULT_CLIENT_VERSION, b'P']);
        transcript.update(&ts.to_le_bytes());
        transcript.update(&encrypt_sk_id.to_le_bytes());
        transcript.update(&nonce_prefix);
        transcript.update(&cid_bind);
        let mut chunk_index = 0;
        let mut total_bytes = 0;
        let pull_start = Instant::now();
        loop {
            stream.set_timeout(self.data_timeout);
            let mut chunk_len = [0; 4];
            stream.read_exact(&mut chunk_len).await.inspect_err(|e| {
                log::error!("stream: failed to read chunk length: {e:?}");
            })?;
            let chunk_len = u32::from_le_bytes(chunk_len);
            if chunk_len == 0 {
                let mut sig = [0; 64];
                stream.read_exact(&mut sig).await.inspect_err(|e| {
                    log::error!("stream: failed to read signature: {e:?}");
                })?;
                let transcript_digest = transcript.finalize().into_bytes();
                self.sign_pk.verify_strict(&transcript_digest, &sig.into()).inspect_err(|_| {
                    log::error!("stream signature verification failed");
                })?;
                return Ok(());
            }
            if chunk_len > MAX_CHUNK_SIZE {
                return Err(Error::ChunkTooLarge { max: MAX_CHUNK_SIZE, got: chunk_len });
            }
            let sealed_len = usize::try_from(chunk_len).unwrap_or_else(|_| {
                unreachable!("chunk_len (max 65536) cannot be too big for usize")
            }) + 16;
            total_bytes += u64::try_from(sealed_len).unwrap_or_else(|_| {
                unreachable!("sealed_len (max 65536 + 16) cannot be too big for u64")
            });
            if total_bytes > self.max_stream_bytes {
                return Err(Error::StreamTooLarge);
            }
            if pull_start.elapsed() > self.max_stream_duration {
                return Err(Error::StreamDurationExceeded);
            }
            let mut sealed = vec![0; sealed_len];
            stream.set_timeout(self.data_timeout);
            stream.read_exact(&mut sealed).await.inspect_err(|e| {
                log::error!("stream: failed to read chunk data: {e:?}");
            })?;
            let nonce = derive_chunk_nonce(nonce_prefix, chunk_index);
            let plain = aead.decrypt(&nonce.into(), sealed.as_slice()).inspect_err(|_| {
                log::error!("stream: aead authentication failed for chunk {chunk_index}");
            })?;
            transcript.update(&chunk_index.to_le_bytes());
            transcript.update(&chunk_len.to_le_bytes());
            transcript.update(&sealed);
            writer.write_all(&plain).await?;
            chunk_index += 1;
        }
    }
}

impl core::fmt::Debug for Client {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let display_secrets = f.alternate();
        let mut s = f.debug_struct("Client");
        s.field("addr", &self.addr)
            .field("timeout", &self.timeout)
            .field("data_timeout", &self.data_timeout)
            .field("ttl", &self.ttl)
            .field("max_stream_bytes", &self.max_stream_bytes)
            .field("max_stream_duration", &self.max_stream_duration);
        if display_secrets {
            let mut id = [0; 16];
            let id_hex = crate::util::hex(&self.encrypt_sk_id.to_le_bytes(), &mut id);
            let mut buf = [0; 64];
            s.field("encrypt_sk", &crate::util::hex(&self.encrypt_sk, &mut buf))
                .field("encrypt_sk_id", &id_hex)
                .field("psk", &crate::util::hex(&self.psk, &mut buf))
                .field("sign_pk", &crate::util::hex(&self.sign_pk.to_bytes(), &mut buf))
                .field("sign_sk", &crate::util::hex(&self.sign_sk.to_bytes(), &mut buf));
        }
        s.finish()
    }
}
