use crate::{
    MAX_CHUNK_SIZE, MAX_FUTURE_SKEW,
    authentication::{
        auth0, auth1, auth2get, auth2store, auth3get, auth3store, compute_cid_bind,
        derive_chunk_nonce, derive_stream_key, new_transcript_hash,
    },
    config::Config,
    error::Error,
    util::Stream,
};
use anyhow::Context;
use blake2::digest::Mac;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20poly1305::{KeyInit, aead::Aead};
use ctutils::CtEq;
use ed25519_dalek::Signer;
use rand::Rng;
use std::{
    io::{self, IsTerminal, Read, Write},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub const DEFAULT_CLIENT_VERSION: u8 = 2;

pub enum Mode<'a> {
    Copy,
    Paste,
    Move,
    Push { cid: &'a str },
    Pull { cid: &'a str },
}

async fn copy_operation(config: &Config, s: &mut Stream, h1: &[u8]) -> Result<(), Error> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is broken")
        .as_secs()
        .to_le_bytes();
    let mut content_with_encrypt_sk_id_and_nonce = vec![0; 32];
    content_with_encrypt_sk_id_and_nonce[..8]
        .copy_from_slice(&config.encrypt_sk_id().to_le_bytes());
    let mut rng = rand::make_rng::<rand::rngs::StdRng>();
    rng.fill_bytes(&mut content_with_encrypt_sk_id_and_nonce[8..32]);
    io::stdin().lock().read_to_end(&mut content_with_encrypt_sk_id_and_nonce)?;
    let opcode = b'S';
    let mut cipher = chacha20::XChaCha20::new(
        &config.encrypt_sk().into(),
        &content_with_encrypt_sk_id_and_nonce[8..32]
            .try_into()
            .expect("8..32 doesn't span 24 bytes. math has died."),
    );
    let ct = &mut content_with_encrypt_sk_id_and_nonce[32..];
    cipher.apply_keystream(ct);
    let signature = config.sign_sk().sign(content_with_encrypt_sk_id_and_nonce.as_slice());
    s.set_timeout(config.data_timeout());
    let h2 = auth2store(config.psk(), h1, opcode, &ts, &signature.to_bytes());
    let h3 = (async || -> io::Result<[u8; 32]> {
        s.write_all(&[opcode]).await?;
        s.write_all(&h2).await?;
        let ciphertext_with_encrypt_sk_id_and_nonce_len =
            content_with_encrypt_sk_id_and_nonce.len() as u64;
        s.write_all(&ciphertext_with_encrypt_sk_id_and_nonce_len.to_le_bytes()).await?;
        s.write_all(&ts).await?;
        s.write_all(&signature.to_bytes()).await?;
        s.write_all(&content_with_encrypt_sk_id_and_nonce).await?;
        s.flush().await?;
        let mut h3 = [0; 32];
        s.read_exact(&mut h3).await?;
        Ok(h3)
    })()
    .await
    .map_err(|e| {
        if matches!(e.kind(), io::ErrorKind::UnexpectedEof | io::ErrorKind::BrokenPipe) {
            Error::ConnectionClosed
        } else {
            e.into()
        }
    })?;
    let wh3 = auth3store(config.psk(), &h2);
    let choice = wh3.ct_ne(&h3);
    if choice.into() {
        return Err(Error::Auth);
    }
    if io::stderr().is_terminal() {
        eprintln!("Sent");
    }
    Ok(())
}

async fn paste_operation(
    config: &Config,
    stream: &mut Stream,
    h1: &[u8],
    is_move: bool,
) -> Result<(), Error> {
    let opcode = if is_move { b'M' } else { b'G' };
    let h2 = auth2get(config.psk(), h1, opcode);
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
    let wh3 = &auth3get(config.psk(), &h2, &ts, &signature)[..];
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
    if elapsed >= config.ttl() {
        return Err(Error::Old);
    }
    if ciphertext_with_encrypt_sk_id_and_nonce_len < 32 {
        return Err(Error::Short(None));
    }
    let full_buf_len = ciphertext_with_encrypt_sk_id_and_nonce_len.try_into().map_err(|_| {
        Error::Large { max: usize::MAX as _, got: ciphertext_with_encrypt_sk_id_and_nonce_len }
    })?;
    let mut ciphertext_with_encrypt_sk_id_and_nonce = vec![0; full_buf_len];
    stream.set_timeout(config.data_timeout());
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
    let choice = encrypt_sk_id.ct_ne(&config.encrypt_sk_id().to_le_bytes());
    if choice.into() {
        let w_encrypt_sk = config.encrypt_sk_id();
        let encrypt_sk = u64::from_le_bytes(encrypt_sk_id);
        return Err(Error::SecretKeyIDMismatch { expected: w_encrypt_sk, actual: encrypt_sk });
    }
    config.sign_pk().verify_strict(
        &ciphertext_with_encrypt_sk_id_and_nonce[..],
        &ed25519_dalek::Signature::from_bytes(&signature),
    )?;
    let nonce = &ciphertext_with_encrypt_sk_id_and_nonce[8..32];
    let mut cipher = chacha20::XChaCha20::new(
        &config.encrypt_sk().into(),
        nonce.try_into().expect("8..32 doesn't span 24 bytes. math has died."),
    );
    let content = &mut ciphertext_with_encrypt_sk_id_and_nonce[32..];
    cipher.apply_keystream(&mut content[..]);
    io::stdout().lock().write_all(content)?;
    io::stdout().lock().flush()?;
    Ok(())
}

async fn push_stream_operation(
    config: &Config,
    stream: &mut Stream,
    h1: &[u8],
    cid: &str,
) -> Result<(), Error> {
    let opcode = b'P';
    let h2 = auth2get(config.psk(), h1, opcode);
    stream.write_all(&[opcode]).await?;
    stream.write_all(&h2).await?;
    stream.flush().await?;
    stream.set_timeout(config.timeout());
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
        derive_stream_key(config.encrypt_sk(), ts, config.encrypt_sk_id(), nonce_prefix, cid_bytes);
    let aead = chacha20poly1305::XChaCha20Poly1305::new(&stream_key.into());
    let cid_bind = compute_cid_bind(config.encrypt_sk(), cid_bytes);
    let mut transcript = new_transcript_hash();
    transcript.update(&[DEFAULT_CLIENT_VERSION, opcode]);
    transcript.update(&ts.to_le_bytes());
    transcript.update(&config.encrypt_sk_id().to_le_bytes());
    transcript.update(&nonce_prefix);
    transcript.update(&cid_bind);
    let mut header = [0; 32];
    header[..8].copy_from_slice(&ts.to_le_bytes());
    header[8..16].copy_from_slice(&config.encrypt_sk_id().to_le_bytes());
    header[16..32].copy_from_slice(&nonce_prefix);
    stream.set_timeout(config.data_timeout());
    stream.write_all(&header).await?;
    stream.flush().await?;
    let mut plain_buf = vec![0; MAX_CHUNK_SIZE as usize];
    let mut chunk_index = 0;
    loop {
        let res = io::stdin().lock().read(&mut plain_buf);
        match res {
            Ok(n) if n > 0 => {
                let nonce = derive_chunk_nonce(nonce_prefix, chunk_index);
                let sealed = aead.encrypt(&nonce.into(), &plain_buf[..n])?;
                let chunk_len = u32::try_from(n).expect("chunk can't be too big for u32");
                transcript.update(&chunk_index.to_le_bytes());
                transcript.update(&chunk_len.to_le_bytes());
                transcript.update(&sealed);
                stream.set_timeout(config.data_timeout());
                stream.write_all(&chunk_len.to_le_bytes()).await?;
                stream.write_all(&sealed).await?;
                stream.flush().await?;
                chunk_index += 1;
            }
            // Ok(0)
            Ok(_) => break,
            Err(e) => {
                if matches!(e.kind(), io::ErrorKind::UnexpectedEof | io::ErrorKind::BrokenPipe) {
                    break;
                }
                return Err(e.into());
            }
        }
    }
    let transcript_digest = transcript.finalize().into_bytes();
    let signature = config.sign_sk().sign(&transcript_digest);
    let end_marker = [0; 4];
    stream.set_timeout(config.data_timeout());
    stream.write_all(&end_marker).await?;
    stream.write_all(&signature.to_bytes()).await?;
    stream.flush().await?;
    if io::stderr().is_terminal() {
        eprintln!("Stream sent");
    }
    Ok(())
}

async fn pull_stream_operation(
    config: &Config,
    stream: &mut Stream,
    h1: &[u8],
    cid: &str,
) -> anyhow::Result<()> {
    let opcode = b'L';
    let h2 = auth2get(config.psk(), h1, opcode);
    stream.write_all(&[opcode]).await?;
    stream.write_all(&h2).await?;
    stream.flush().await?;
    let cid_bytes = cid.as_bytes();
    stream.clear_timeout();
    let mut header = [0; 32];
    stream.read_exact(&mut header).await.context("stream: failed to read header")?;
    let ts = u64::from_le_bytes(header[..8].try_into().expect("0..8 is 8 bytes lol"));
    let encrypt_sk_id = u64::from_le_bytes(header[8..16].try_into().expect("8..16 is 8 bytes lol"));
    let nonce_prefix = header[16..32].try_into().expect("16..32 is 16 bytes lol");
    let now = SystemTime::now();
    let ts_val = UNIX_EPOCH + Duration::from_secs(ts);
    match now.duration_since(ts_val) {
        Ok(elapsed) => {
            if elapsed > config.ttl() {
                return Err(Error::Old.into());
            }
        }
        Err(skew) => {
            if skew.duration() > MAX_FUTURE_SKEW {
                return Err(Error::Future.into());
            }
        }
    }
    if encrypt_sk_id != config.encrypt_sk_id() {
        return Err(Error::SecretKeyIDMismatch {
            expected: config.encrypt_sk_id(),
            actual: encrypt_sk_id,
        }
        .into());
    }
    let stream_key =
        derive_stream_key(config.encrypt_sk(), ts, encrypt_sk_id, nonce_prefix, cid_bytes);
    let aead = chacha20poly1305::XChaCha20Poly1305::new(&stream_key.into());
    let cid_bind = compute_cid_bind(config.encrypt_sk(), cid_bytes);
    let mut transcript = new_transcript_hash();
    transcript.update(&[DEFAULT_CLIENT_VERSION, b'P']);
    transcript.update(&ts.to_le_bytes());
    transcript.update(&encrypt_sk_id.to_le_bytes());
    transcript.update(&nonce_prefix);
    transcript.update(&cid_bind);
    let mut chunk_index = 0;
    let mut total_bytes = 0;
    let pull_start = SystemTime::now();
    loop {
        stream.set_timeout(config.data_timeout());
        let mut chunk_len = [0; 4];
        stream.read_exact(&mut chunk_len).await.context("stream: failed to read chunk length")?;
        let chunk_len = u32::from_le_bytes(chunk_len);
        if chunk_len == 0 {
            let mut sig = [0; 64];
            stream.read_exact(&mut sig).await.context("stream: failed to read signature")?;
            let transcript_digest = transcript.finalize().into_bytes();
            config
                .sign_pk()
                .verify_strict(&transcript_digest, &sig.into())
                .context("stream signature verification failed")?;
            return Ok(());
        }
        if chunk_len > MAX_CHUNK_SIZE {
            return Err(Error::ChunkTooLarge { max: MAX_CHUNK_SIZE, got: chunk_len }.into());
        }
        let sealed_len = usize::try_from(chunk_len).expect("chunk_len is too big for usize") + 16;
        total_bytes += u64::try_from(sealed_len).expect("sealed_len is too big for u64");
        if total_bytes > config.max_stream_bytes() {
            return Err(Error::StreamTooLarge.into());
        }
        let elapsed = SystemTime::now().duration_since(pull_start).expect("system clock is broken");
        if elapsed > config.max_stream_duration() {
            return Err(Error::StreamDurationExceeded.into());
        }
        let mut sealed = vec![0; sealed_len];
        stream.set_timeout(config.data_timeout());
        stream.read_exact(&mut sealed).await.context("stream: failed to read chunk data")?;
        let nonce = derive_chunk_nonce(nonce_prefix, chunk_index);
        let plain = aead.decrypt(&nonce.into(), sealed.as_slice()).with_context(|| {
            format!("stream: aead authentication failed for chunk {chunk_index}")
        })?;
        transcript.update(&chunk_index.to_le_bytes());
        transcript.update(&chunk_len.to_le_bytes());
        transcript.update(&sealed);
        io::stdout().lock().write_all(&plain)?;
        chunk_index += 1;
    }
}

pub async fn run(config: Config, mode: Mode<'_>) -> anyhow::Result<()> {
    let psk = config.psk();
    let fut = tokio::net::TcpStream::connect(config.connect());
    let conn = tokio::time::timeout(config.timeout(), fut).await.map_err(io::Error::from)??;
    let mut stream = Stream::new(conn);
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
            anyhow::anyhow!(
                "the server rejected the connection - check that it is running the same klip \
                 version or retry later"
            )
        } else {
            anyhow::anyhow!(Error::ProtocolUnsupported)
        }
    })?;
    if rbuf[0] != DEFAULT_CLIENT_VERSION {
        return Err(Error::IncompatibleVersions {
            client: DEFAULT_CLIENT_VERSION,
            server: rbuf[0],
        }
        .into());
    }
    let r2 = &rbuf[1..33];
    let h1 = &rbuf[33..65];
    let wh1 = &auth1(psk, DEFAULT_CLIENT_VERSION, &h0, r2)[..];
    let choice = wh1.ct_ne(h1);
    if choice.into() {
        return Err(Error::Auth.into());
    }
    match mode {
        Mode::Copy => copy_operation(&config, &mut stream, h1).await?,
        Mode::Paste => paste_operation(&config, &mut stream, h1, false).await?,
        Mode::Move => paste_operation(&config, &mut stream, h1, true).await?,
        Mode::Push { cid } => push_stream_operation(&config, &mut stream, h1, cid).await?,
        Mode::Pull { cid } => pull_stream_operation(&config, &mut stream, h1, cid).await?,
    }
    Ok(())
}
