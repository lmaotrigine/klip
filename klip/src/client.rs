use crate::{
    authentication::{auth0, auth1, auth2get, auth2store, auth3get, auth3store},
    config::Config,
    error::Error,
    util::{is_a_tty, Stream},
};
use crypto_common::constant_time::ConstantTimeEq;
use rand_core::RngCore;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[macro_export]
macro_rules! default_client_version {
    () => {
        1
    };
}

const DEFAULT_CLIENT_VERSION: u8 = crate::default_client_version!();

async fn copy_operation(config: &Config, s: &mut Stream, h1: &[u8]) -> Result<(), Error> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is broken")
        .as_secs()
        .to_le_bytes();
    let mut content_with_encrypt_sk_id_and_nonce = vec![0; 32];
    content_with_encrypt_sk_id_and_nonce[..8]
        .copy_from_slice(&config.encrypt_sk_id().to_le_bytes());
    let mut rng = rand_core::OsRng;
    rng.fill_bytes(&mut content_with_encrypt_sk_id_and_nonce[8..32]);
    io::stdin()
        .lock()
        .read_to_end(&mut content_with_encrypt_sk_id_and_nonce)?;
    let opcode = b'S';
    let mut cipher = xchacha20::XChaCha20::new(
        &config.encrypt_sk(),
        &content_with_encrypt_sk_id_and_nonce[8..32]
            .try_into()
            .expect("8..32 doesn't span 24 bytes. math has died."),
    );
    let ct = &mut content_with_encrypt_sk_id_and_nonce[32..];
    cipher.apply_keystream(ct);
    assert_eq!(
        &content_with_encrypt_sk_id_and_nonce[0..8],
        &config.encrypt_sk_id().to_le_bytes()
    );
    let signature = config
        .sign_sk()
        .sign(content_with_encrypt_sk_id_and_nonce.as_slice());
    s.set_timeout(config.data_timeout());
    let h2 = auth2store(config.psk(), h1, opcode, &ts, &signature.to_bytes());
    s.write_all(&[opcode]).await?;
    s.write_all(h2.as_bytes()).await?;
    let ciphertext_with_encrypt_sk_id_and_nonce_len =
        content_with_encrypt_sk_id_and_nonce.len() as u64;
    s.write_all(&ciphertext_with_encrypt_sk_id_and_nonce_len.to_le_bytes())
        .await?;
    s.write_all(&ts).await?;
    s.write_all(&signature.to_bytes()).await?;
    s.write_all(&content_with_encrypt_sk_id_and_nonce).await?;
    s.flush().await?;
    let mut rbuf = [0; 32];
    s.read_exact(&mut rbuf).await.map_err(|e| {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            Error::MaybeIncompatibleVersion
        } else {
            e.into()
        }
    })?;
    let h3 = &rbuf[..32];
    let wh3 = auth3store(config.psk(), h2.as_bytes());
    if wh3.as_bytes().ct_eq(h3).to_u8() != 1 {
        return Err(Error::Auth);
    }
    if is_a_tty(true) {
        eprintln!("Sent");
    }
    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
async fn paste_operation(
    config: &Config,
    stream: &mut Stream,
    h1: &[u8],
    is_move: bool,
) -> Result<(), Error> {
    let opcode = if is_move { b'M' } else { b'G' };
    let h2 = auth2get(config.psk(), h1, opcode);
    stream.write_all(&[opcode]).await?;
    stream.write_all(h2.as_bytes()).await?;
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
    let wh3 = auth3get(config.psk(), h2.as_bytes(), &ts, &signature);
    if wh3.as_bytes().ct_eq(h3).to_u8() != 1 {
        return Err(Error::Auth);
    }
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH + Duration::from_secs(u64::from_le_bytes(ts)))
        .expect("clock is broken");
    if elapsed >= config.ttl() {
        return Err(Error::Old);
    }
    if ciphertext_with_encrypt_sk_id_and_nonce_len < 32 {
        return Err(Error::Short);
    }
    let mut ciphertext_with_encrypt_sk_id_and_nonce =
        Vec::with_capacity(ciphertext_with_encrypt_sk_id_and_nonce_len as usize);
    stream.set_timeout(config.data_timeout());
    stream
        .read_to_end(&mut ciphertext_with_encrypt_sk_id_and_nonce)
        .await
        .map_err(|e| {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                Error::MaybeIncompatibleVersion
            } else {
                e.into()
            }
        })?;
    let encrypt_sk_id = {
        let c = &ciphertext_with_encrypt_sk_id_and_nonce[..8];
        [c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]
    };
    if encrypt_sk_id
        .ct_eq(&config.encrypt_sk_id().to_le_bytes())
        .to_u8()
        != 1
    {
        let w_encrypt_sk = config.encrypt_sk_id();
        let encrypt_sk = u64::from_le_bytes(encrypt_sk_id);
        return Err(Error::SecretKeyIDMismatch {
            expected: w_encrypt_sk,
            actual: encrypt_sk,
        });
    }
    config.sign_pk().verify_strict(
        &ciphertext_with_encrypt_sk_id_and_nonce[..],
        &ed25519::Signature::from_bytes(&signature)?,
    )?;
    let nonce = &ciphertext_with_encrypt_sk_id_and_nonce[8..32];
    let mut cipher = xchacha20::XChaCha20::new(
        &config.encrypt_sk(),
        nonce
            .try_into()
            .expect("8..32 doesn't span 24 bytes. math has died."),
    );
    let content = &mut ciphertext_with_encrypt_sk_id_and_nonce[32..];
    cipher.apply_keystream(&mut content[..]);
    io::stdout().lock().write_all(content)?;
    io::stdout().lock().flush()?;
    Ok(())
}

pub async fn run(config: Config, is_copy: bool, is_move: bool) -> Result<(), Error> {
    let psk = config.psk();
    let conn = TcpStream::connect_timeout(&config.connect(), config.timeout())?;
    let s = tokio::net::TcpStream::from_std(conn)?;
    let mut stream = Stream::new(s);
    let mut r = [0; 32];
    let mut rng = rand_core::OsRng;
    rng.fill_bytes(&mut r);
    let h0 = auth0(psk, DEFAULT_CLIENT_VERSION, &r);
    stream.write_all(&[DEFAULT_CLIENT_VERSION]).await?;
    stream.write_all(&r).await?;
    stream.write_all(h0.as_bytes()).await?;
    stream.flush().await?;
    let mut rbuf = [0; 65];
    stream.read_exact(&mut rbuf).await.map_err(|_| {
        if rbuf.iter().position(|&b| b == 0).is_some_and(|p| p < 2) {
            io::Error::new(
                io::ErrorKind::ConnectionRefused,
                "the server rejected the connection - check that it is running the same klip \
                 version or retry later",
            )
            .into()
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
    let wh1 = auth1(psk, DEFAULT_CLIENT_VERSION, h0.as_bytes(), r2);
    if wh1.as_bytes().ct_eq(h1).to_u8() != 1 {
        return Err(Error::Auth);
    }
    if is_copy {
        copy_operation(&config, &mut stream, h1).await
    } else {
        paste_operation(&config, &mut stream, h1, is_move).await
    }
}
