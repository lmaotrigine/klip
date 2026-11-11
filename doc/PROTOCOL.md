# klip protocol

klip's protocol borrows extensively from the work of Frank Denis.

## Common definitions

```text
k: pre-shared key
ek: 256-bit symmetric encryption key
ekid: encryption key id encoded as a 64-bit little endian integer
m: plaintext
n: random 192-bit nonce
ct: XChaCha20 ek,n (m)
Hk,s: BLAKE2b(domain="KLIP", key=k, salt=s, size=32)
Len(x): x encoded as a 64-bit little endian integer
r: random 256-bit client nonce
r': random 256-bit server nonce
ts: Unix timestamp as a 64-bit little endian integer
Sig: Ed25519
v: 2
```

## Copy

```text
-> v || r || h0
h0 := Hk,0(v || r)

<- v || r' || h1
h1 := Hk,1(v || r' || h0)

-> 'S' || h2 || Len(ekid || n || ct) || ts || s || ekid || n || ct
s := Sig(ekid || n || ct)
h2 := Hk,2(h1 || 'S' || ts || s)

<- Hk,3(h2)
```

## Move/Paste

```text
Move:  opcode := 'M'
Paste: opcode := 'G'

-> v || r || h0
h0 := Hk,0(v || r)

<- v || r' || h1
h1 := Hk,1(v || r' || h0)

-> opcode || h2
h2 := Hk,2(h1 || opcode)

<- Hk,3(h2 || ts || s) || Len(ekid || n || ct) || ts || s || ekid || n || ct
s := Sig(ekid || n || ct)
```

## Streaming

### Additional definitions

```text
AEAD: XChaCha20-Poly1305
np: random 128-bit nonce prefix (also serves as stream identifier)
stream_key: BLAKE2b-256(key=ek, person="k-v2-stream-key",
                        input=ts || ekid || np || uint16(len(cid)) || cid)
chunk_nonce(i): np || uint64_le(i)   (24 bytes)
cid_bind: BLAKE2b-256(key=ek, person="k-v2-cid-bind",
          input=uint16(len(cid)) || cid  if cid present
                32 zero bytes            if cid absent
          )
```

### Push (sender)

```text
-> v || r || h0                                                     (handshake)
<- v || r' || h1

-> 'P' || h2                                                      (opcode auth)
h2 := Hk,2(h1 || 'P')

<- status                   (1 byte: 0x01=accepted, 0x00=no pullers, 0x02=busy)

-> ts || ekid || np                                   (stream header, 32 bytes)

-> uint32_le(len) || AEAD.encrypt(stream_key, chunk_nonce(0), chunk_0)
-> uint32_le(len) || AEAD.encrypt(stream_key, chunk_nonce(1), chunk_1)
...                                                 (data frames, len=1..65536)

-> uint32_le(0) || Sig(transcript_hash)                             (end frame)
```

The server rejects the push immediately if no pullers are waiting or another
push is already active. The sender receives a distinct status code for each
case and can report a meaningful error without transmitting any data.

### Pull (receiver)

```text
-> v || r || h0                                                     (handshake)
<- v || r' || h1

-> 'L' || h2                                                      (opcode auth)
h2 := Hk,2(h1 || 'L')

<- status                (1 byte: 0x01=accepted, 0x00=stream active, 0x02=full)

<- ts || ekid || np                                             (stream header)
<- uint32_le(len) || encrypted_chunk                              (data frames)
...
<- uint32_le(0) || signature                                        (end frame)
```

The server rejects the pull immediately if a push is already in progress or if
the maximum number of waiting pullers has been reached.

The transcript hash contains:

```text
transcript_hash := BLAKE2b-256(person="kl-v2-transcript",
  v(1) || 'P'(1) || ts(8) || ekid(8) || np(16) || cid_bind(32)
  || for each chunk: chunk_index(8) || chunk_len(4) || encrypted_chunk(len+16)
)
```

All integers are little-endian. The content identifier (`--cid`) is never
transmitted over the wire; it is bound into the key derivation and the
transcript hash to ensure both sides agree on the same label.
