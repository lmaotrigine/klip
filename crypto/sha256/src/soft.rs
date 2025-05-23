use crate::consts::K;

#[inline(always)]
const fn shr(v: [u32; 4], o: u32) -> [u32; 4] {
    [v[0] >> o, v[1] >> o, v[2] >> o, v[3] >> o]
}

#[inline(always)]
const fn shl(v: [u32; 4], o: u32) -> [u32; 4] {
    [v[0] << o, v[1] << o, v[2] << o, v[3] << o]
}

#[inline(always)]
const fn or(v: [u32; 4], o: [u32; 4]) -> [u32; 4] {
    [v[0] | o[0], v[1] | o[1], v[2] | o[2], v[3] | o[3]]
}

#[inline(always)]
const fn xor(v: [u32; 4], o: [u32; 4]) -> [u32; 4] {
    [v[0] ^ o[0], v[1] ^ o[1], v[2] ^ o[2], v[3] ^ o[3]]
}

#[inline(always)]
const fn add(v: [u32; 4], o: [u32; 4]) -> [u32; 4] {
    [
        v[0].wrapping_add(o[0]),
        v[1].wrapping_add(o[1]),
        v[2].wrapping_add(o[2]),
        v[3].wrapping_add(o[3]),
    ]
}

#[inline(always)]
fn add_round_const(mut a: [u32; 4], i: usize) -> [u32; 4] {
    #[cfg_attr(
        any(target_arch = "x86", target_arch = "x86_64"),
        allow(clippy::missing_const_for_fn)
    )]
    fn k(i: usize, j: usize) -> u32 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        use core::ptr::read as r;
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        use core::ptr::read_volatile as r;

        unsafe { r(K.as_ptr().add(4 * i + j)) }
    }

    a[3] = a[3].wrapping_add(k(i, 0));
    a[2] = a[2].wrapping_add(k(i, 1));
    a[1] = a[1].wrapping_add(k(i, 2));
    a[0] = a[0].wrapping_add(k(i, 3));
    a
}

const fn sha256load(v2: [u32; 4], v3: [u32; 4]) -> [u32; 4] {
    [v3[3], v2[0], v2[1], v2[2]]
}

const fn sha256swap(v0: [u32; 4]) -> [u32; 4] {
    [v0[2], v0[3], v0[0], v0[1]]
}

const fn sha256msg1(v0: [u32; 4], v1: [u32; 4]) -> [u32; 4] {
    #[inline]
    const fn sigma0x4(x: [u32; 4]) -> [u32; 4] {
        let t1 = or(shr(x, 7), shl(x, 25));
        let t2 = or(shr(x, 18), shl(x, 14));
        let t3 = shr(x, 3);
        xor(xor(t1, t2), t3)
    }
    add(v0, sigma0x4(sha256load(v0, v1)))
}

const fn sha256msg2(v4: [u32; 4], v3: [u32; 4]) -> [u32; 4] {
    macro_rules! sigma1 {
        ($a:expr) => {
            $a.rotate_right(17) ^ $a.rotate_right(19) ^ ($a >> 10)
        };
    }
    let [x3, x2, x1, x0] = v4;
    let [w15, w14, _, _] = v3;
    let w16 = x0.wrapping_add(sigma1!(w14));
    let w17 = x1.wrapping_add(sigma1!(w15));
    let w18 = x2.wrapping_add(sigma1!(w16));
    let w19 = x3.wrapping_add(sigma1!(w17));
    [w19, w18, w17, w16]
}

const fn sha256_digest_round_x2(cdgh: [u32; 4], abef: [u32; 4], wk: [u32; 4]) -> [u32; 4] {
    macro_rules! big_sigma0 {
        ($a:expr) => {
            ($a.rotate_right(2) ^ $a.rotate_right(13) ^ $a.rotate_right(22))
        };
    }
    macro_rules! big_sigma1 {
        ($a:expr) => {
            ($a.rotate_right(6) ^ $a.rotate_right(11) ^ $a.rotate_right(25))
        };
    }
    macro_rules! bool3ary_202 {
        ($a:expr, $b:expr, $c:expr) => {
            $c ^ ($a & ($b ^ $c))
        };
    }
    macro_rules! bool3ary_232 {
        ($a:expr, $b:expr, $c:expr) => {
            ($a & $b) ^ ($a & $c) ^ ($b & $c)
        };
    }
    let [_, _, wk1, wk0] = wk;
    let [a0, b0, e0, f0] = abef;
    let [c0, d0, g0, h0] = cdgh;
    let x0 = big_sigma1!(e0)
        .wrapping_add(bool3ary_202!(e0, f0, g0))
        .wrapping_add(wk0)
        .wrapping_add(h0);
    let y0 = big_sigma0!(a0).wrapping_add(bool3ary_232!(a0, b0, c0));
    let (a1, b1, c1, d1, e1, f1, g1, h1) = (
        x0.wrapping_add(y0),
        a0,
        b0,
        c0,
        x0.wrapping_add(d0),
        e0,
        f0,
        g0,
    );
    let x1 = big_sigma1!(e1)
        .wrapping_add(bool3ary_202!(e1, f1, g1))
        .wrapping_add(wk1)
        .wrapping_add(h1);
    let y1 = big_sigma0!(a1).wrapping_add(bool3ary_232!(a1, b1, c1));
    let (a2, b2, _, _, e2, f2, _, _) = (
        x1.wrapping_add(y1),
        a1,
        b1,
        c1,
        x1.wrapping_add(d1),
        e1,
        f1,
        g1,
    );
    [a2, b2, e2, f2]
}

const fn schedule(v0: [u32; 4], v1: [u32; 4], v2: [u32; 4], v3: [u32; 4]) -> [u32; 4] {
    let t1 = sha256msg1(v0, v1);
    let t2 = sha256load(v2, v3);
    let t3 = add(t1, t2);
    sha256msg2(t3, v3)
}

macro_rules! rounds4 {
    ($abef:ident, $cdgh:ident, $rest:expr, $i:expr) => {{
        let t1 = add_round_const($rest, $i);
        $cdgh = sha256_digest_round_x2($cdgh, $abef, t1);
        let t2 = sha256swap(t1);
        $abef = sha256_digest_round_x2($abef, $cdgh, t2);
    }};
}

macro_rules! schedule_rounds4 {
    ($abef:ident, $cdgh:ident, $w0:expr, $w1:expr, $w2:expr, $w3:expr, $w4:expr, $i:expr) => {{
        $w4 = schedule($w0, $w1, $w2, $w3);
        rounds4!($abef, $cdgh, $w4, $i);
    }};
}

#[allow(clippy::many_single_char_names)]
fn sha256_digest_block_u32(state: &mut [u32; 8], block: [u32; 16]) {
    let mut abef = [state[0], state[1], state[4], state[5]];
    let mut cdgh = [state[2], state[3], state[6], state[7]];
    let mut w0 = [block[3], block[2], block[1], block[0]];
    let mut w1 = [block[7], block[6], block[5], block[4]];
    let mut w2 = [block[11], block[10], block[9], block[8]];
    let mut w3 = [block[15], block[14], block[13], block[12]];
    let mut w4;
    rounds4!(abef, cdgh, w0, 0);
    rounds4!(abef, cdgh, w1, 1);
    rounds4!(abef, cdgh, w2, 2);
    rounds4!(abef, cdgh, w3, 3);
    schedule_rounds4!(abef, cdgh, w0, w1, w2, w3, w4, 4);
    schedule_rounds4!(abef, cdgh, w1, w2, w3, w4, w0, 5);
    schedule_rounds4!(abef, cdgh, w2, w3, w4, w0, w1, 6);
    schedule_rounds4!(abef, cdgh, w3, w4, w0, w1, w2, 7);
    schedule_rounds4!(abef, cdgh, w4, w0, w1, w2, w3, 8);
    schedule_rounds4!(abef, cdgh, w0, w1, w2, w3, w4, 9);
    schedule_rounds4!(abef, cdgh, w1, w2, w3, w4, w0, 10);
    schedule_rounds4!(abef, cdgh, w2, w3, w4, w0, w1, 11);
    schedule_rounds4!(abef, cdgh, w3, w4, w0, w1, w2, 12);
    schedule_rounds4!(abef, cdgh, w4, w0, w1, w2, w3, 13);
    schedule_rounds4!(abef, cdgh, w0, w1, w2, w3, w4, 14);
    schedule_rounds4!(abef, cdgh, w1, w2, w3, w4, w0, 15);
    let [a, b, e, f] = abef;
    let [c, d, g, h] = cdgh;
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

#[inline(always)]
fn to_u32s(block: &[u8; 64]) -> [u32; 16] {
    let mut res = [0; 16];
    for (src, dst) in block.chunks_exact(4).zip(res.iter_mut()) {
        *dst = u32::from_be_bytes(src.try_into().expect("math has died"));
    }
    res
}

#[allow(unused)]
pub fn compress(state: &mut [u32; 8], blocks: &[[u8; 64]]) {
    for block in blocks.iter().map(to_u32s) {
        sha256_digest_block_u32(state, block);
    }
}
