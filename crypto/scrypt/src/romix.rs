#[allow(clippy::many_single_char_names)]
pub fn scrypt_ro_mix(b: &mut [u8], v: &mut [u8], t: &mut [u8], n: usize) {
    fn integerify(x: &[u8], n: usize) -> usize {
        let mask = n - 1;
        let t = u32::from_le_bytes(x[x.len() - 64..x.len() - 60].try_into().unwrap());
        (t as usize) & mask
    }
    let len = b.len();
    for chunk in v.chunks_mut(len) {
        chunk.copy_from_slice(b);
        scrypt_block_mix(chunk, b);
    }
    for _ in 0..n {
        let j = integerify(b, n);
        xor(b, &v[j * len..(j + 1) * len], t);
        scrypt_block_mix(t, b);
    }
}

fn scrypt_block_mix(input: &[u8], output: &mut [u8]) {
    use crate::salsa::Salsa;
    let mut x = [0; 64];
    x.copy_from_slice(&input[input.len() - 64..]);
    let mut t = [0; 64];
    for (i, chunk) in input.chunks(64).enumerate() {
        xor(&x, chunk, &mut t);
        let mut t2 = [0; 16];
        for (c, b) in t.chunks_exact(4).zip(t2.iter_mut()) {
            *b = u32::from_le_bytes(c.try_into().unwrap());
        }
        Salsa::from_raw_state(t2).write_keystream_block(&mut x);
        let pos = if i % 2 == 0 {
            (i / 2) * 64
        } else {
            (i / 2) * 64 + input.len() / 2
        };
        output[pos..pos + 64].copy_from_slice(&x);
    }
}

fn xor(x: &[u8], y: &[u8], output: &mut [u8]) {
    for ((out, &x_i), &y_i) in output.iter_mut().zip(x.iter()).zip(y.iter()) {
        *out = x_i ^ y_i;
    }
}
