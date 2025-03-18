use crate::{
    core::{count_high, count_low, final_block, flag_word, input_assertions},
    utils::{as_arrays, as_arrays_mut},
    BLOCK_BYTES, IV,
};
#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm256_add_epi64, _mm256_alignr_epi8, _mm256_blend_epi32,
    _mm256_broadcastsi128_si256, _mm256_loadu_si256, _mm256_or_si256, _mm256_permute4x64_epi64,
    _mm256_setr_epi64x, _mm256_shuffle_epi32, _mm256_slli_epi64, _mm256_srli_epi64,
    _mm256_storeu_si256, _mm256_unpackhi_epi64, _mm256_unpacklo_epi64, _mm256_xor_si256,
    _mm_loadu_si128,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, __m256i, _mm256_add_epi64, _mm256_alignr_epi8, _mm256_blend_epi32,
    _mm256_broadcastsi128_si256, _mm256_loadu_si256, _mm256_or_si256, _mm256_permute4x64_epi64,
    _mm256_setr_epi64x, _mm256_shuffle_epi32, _mm256_slli_epi64, _mm256_srli_epi64,
    _mm256_storeu_si256, _mm256_unpackhi_epi64, _mm256_unpacklo_epi64, _mm256_xor_si256,
    _mm_loadu_si128,
};

pub const DEGREE: usize = 4;

#[inline(always)]
#[allow(clippy::cast_ptr_alignment, clippy::ptr_as_ptr)]
unsafe fn loadu(src: *const [u64; DEGREE]) -> __m256i {
    // this is an unaligned load, so the cast is safe.
    _mm256_loadu_si256(src as *const __m256i)
}

#[inline(always)]
#[allow(clippy::cast_ptr_alignment, clippy::ptr_as_ptr)]
unsafe fn storeu(src: __m256i, dest: *mut [u64; DEGREE]) {
    // this is an unaligned store, so the cast is safe.
    _mm256_storeu_si256(dest as *mut __m256i, src);
}

#[inline(always)]
#[allow(clippy::cast_ptr_alignment, clippy::ptr_as_ptr)]
unsafe fn loadu_128(mem_addr: &[u8; 16]) -> __m128i {
    _mm_loadu_si128(mem_addr.as_ptr() as *const __m128i)
}

#[inline(always)]
unsafe fn add(a: __m256i, b: __m256i) -> __m256i {
    _mm256_add_epi64(a, b)
}

#[inline(always)]
unsafe fn xor(a: __m256i, b: __m256i) -> __m256i {
    _mm256_xor_si256(a, b)
}

#[inline(always)]
#[allow(clippy::cast_possible_wrap)]
unsafe fn set4(a: u64, b: u64, c: u64, d: u64) -> __m256i {
    _mm256_setr_epi64x(a as i64, b as i64, c as i64, d as i64)
}

macro_rules! _mm_shuffle {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

#[inline(always)]
unsafe fn rot32(x: __m256i) -> __m256i {
    _mm256_or_si256(_mm256_srli_epi64(x, 32), _mm256_slli_epi64(x, 64 - 32))
}

#[inline(always)]
unsafe fn rot24(x: __m256i) -> __m256i {
    _mm256_or_si256(_mm256_srli_epi64(x, 24), _mm256_slli_epi64(x, 64 - 24))
}

#[inline(always)]
unsafe fn rot16(x: __m256i) -> __m256i {
    _mm256_or_si256(_mm256_srli_epi64(x, 16), _mm256_slli_epi64(x, 64 - 16))
}

#[inline(always)]
unsafe fn rot63(x: __m256i) -> __m256i {
    _mm256_or_si256(_mm256_srli_epi64(x, 63), _mm256_slli_epi64(x, 64 - 63))
}

#[inline(always)]
unsafe fn g1(a: &mut __m256i, b: &mut __m256i, c: &mut __m256i, d: &mut __m256i, m: &mut __m256i) {
    *a = add(*a, *m);
    *a = add(*a, *b);
    *d = xor(*d, *a);
    *d = rot32(*d);
    *c = add(*c, *d);
    *b = xor(*b, *c);
    *b = rot24(*b);
}

#[inline(always)]
unsafe fn g2(a: &mut __m256i, b: &mut __m256i, c: &mut __m256i, d: &mut __m256i, m: &mut __m256i) {
    *a = add(*a, *m);
    *a = add(*a, *b);
    *d = xor(*d, *a);
    *d = rot16(*d);
    *c = add(*c, *d);
    *b = xor(*b, *c);
    *b = rot63(*b);
}

#[inline(always)]
unsafe fn diagonalize(a: &mut __m256i, _b: &mut __m256i, c: &mut __m256i, d: &mut __m256i) {
    *a = _mm256_permute4x64_epi64(*a, _mm_shuffle!(2, 1, 0, 3));
    *d = _mm256_permute4x64_epi64(*d, _mm_shuffle!(1, 0, 3, 2));
    *c = _mm256_permute4x64_epi64(*c, _mm_shuffle!(0, 3, 2, 1));
}

#[inline(always)]
unsafe fn undiagonalize(a: &mut __m256i, _b: &mut __m256i, c: &mut __m256i, d: &mut __m256i) {
    *a = _mm256_permute4x64_epi64(*a, _mm_shuffle!(0, 3, 2, 1));
    *d = _mm256_permute4x64_epi64(*d, _mm_shuffle!(1, 0, 3, 2));
    *c = _mm256_permute4x64_epi64(*c, _mm_shuffle!(2, 1, 0, 3));
}

#[inline(always)]
#[allow(clippy::too_many_lines)]
unsafe fn compress_block(
    block: &[u8; BLOCK_BYTES],
    words: &mut [u64; 8],
    count: u128,
    last_block: u64,
    last_node: u64,
) {
    let (words_low, words_high) = as_arrays_mut!(words, DEGREE, DEGREE);
    let (iv_low, iv_high) = as_arrays!(&IV, DEGREE, DEGREE);
    let mut a = loadu(words_low);
    let mut b = loadu(words_high);
    let mut c = loadu(iv_low);
    let flags = set4(count_low(count), count_high(count), last_block, last_node);
    let mut d = xor(loadu(iv_high), flags);
    let msg_chunks = as_arrays!(block, 16, 16, 16, 16, 16, 16, 16, 16);
    let m0 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.0));
    let m1 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.1));
    let m2 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.2));
    let m3 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.3));
    let m4 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.4));
    let m5 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.5));
    let m6 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.6));
    let m7 = _mm256_broadcastsi128_si256(loadu_128(msg_chunks.7));
    let iv0 = a;
    let iv1 = b;
    // round 1
    let mut t0 = _mm256_unpacklo_epi64(m0, m1);
    let mut t1 = _mm256_unpacklo_epi64(m2, m3);
    let mut b0 = _mm256_blend_epi32(t0, t1, 0xf0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpackhi_epi64(m0, m1);
    t1 = _mm256_unpackhi_epi64(m2, m3);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    diagonalize(&mut a, &mut b, &mut c, &mut d);
    t0 = _mm256_unpacklo_epi64(m7, m4);
    t1 = _mm256_unpacklo_epi64(m5, m6);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpackhi_epi64(m7, m4);
    t1 = _mm256_unpackhi_epi64(m5, m6);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    undiagonalize(&mut a, &mut b, &mut c, &mut d);
    // round 2
    t0 = _mm256_unpacklo_epi64(m7, m2);
    t1 = _mm256_unpackhi_epi64(m4, m6);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpacklo_epi64(m5, m4);
    t1 = _mm256_alignr_epi8(m3, m7, 8);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    diagonalize(&mut a, &mut b, &mut c, &mut d);
    t0 = _mm256_unpackhi_epi64(m2, m0);
    t1 = _mm256_blend_epi32(m5, m0, 0x33);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_alignr_epi8(m6, m1, 8);
    t1 = _mm256_blend_epi32(m3, m1, 0x33);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    undiagonalize(&mut a, &mut b, &mut c, &mut d);
    // round 3
    t0 = _mm256_alignr_epi8(m6, m5, 8);
    t1 = _mm256_unpackhi_epi64(m2, m7);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpacklo_epi64(m4, m0);
    t1 = _mm256_blend_epi32(m6, m1, 0x33);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    diagonalize(&mut a, &mut b, &mut c, &mut d);
    t0 = _mm256_alignr_epi8(m5, m4, 8);
    t1 = _mm256_unpackhi_epi64(m1, m3);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpacklo_epi64(m2, m7);
    t1 = _mm256_blend_epi32(m0, m3, 0x33);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    undiagonalize(&mut a, &mut b, &mut c, &mut d);
    // round 4
    t0 = _mm256_unpackhi_epi64(m3, m1);
    t1 = _mm256_unpackhi_epi64(m6, m5);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpackhi_epi64(m4, m0);
    t1 = _mm256_unpacklo_epi64(m6, m7);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    diagonalize(&mut a, &mut b, &mut c, &mut d);
    t0 = _mm256_alignr_epi8(m1, m7, 8);
    t1 = _mm256_shuffle_epi32(m2, _mm_shuffle!(1, 0, 3, 2));
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpacklo_epi64(m4, m3);
    t1 = _mm256_unpacklo_epi64(m5, m0);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    undiagonalize(&mut a, &mut b, &mut c, &mut d);
    // round 5
    t0 = _mm256_unpackhi_epi64(m4, m2);
    t1 = _mm256_unpacklo_epi64(m1, m5);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_blend_epi32(m3, m0, 0x33);
    t1 = _mm256_blend_epi32(m7, m2, 0x33);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    diagonalize(&mut a, &mut b, &mut c, &mut d);
    t0 = _mm256_alignr_epi8(m7, m1, 8);
    t1 = _mm256_alignr_epi8(m3, m5, 8);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpackhi_epi64(m6, m0);
    t1 = _mm256_unpacklo_epi64(m6, m4);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    undiagonalize(&mut a, &mut b, &mut c, &mut d);
    // round 6
    t0 = _mm256_unpacklo_epi64(m1, m3);
    t1 = _mm256_unpacklo_epi64(m0, m4);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpacklo_epi64(m6, m5);
    t1 = _mm256_unpackhi_epi64(m5, m1);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    diagonalize(&mut a, &mut b, &mut c, &mut d);
    t0 = _mm256_alignr_epi8(m2, m0, 8);
    t1 = _mm256_unpackhi_epi64(m3, m7);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpackhi_epi64(m4, m6);
    t1 = _mm256_alignr_epi8(m7, m2, 8);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    undiagonalize(&mut a, &mut b, &mut c, &mut d);
    // round 7
    t0 = _mm256_blend_epi32(m0, m6, 0x33);
    t1 = _mm256_unpacklo_epi64(m7, m2);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpackhi_epi64(m2, m7);
    t1 = _mm256_alignr_epi8(m5, m6, 8);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    diagonalize(&mut a, &mut b, &mut c, &mut d);
    t0 = _mm256_unpacklo_epi64(m4, m0);
    t1 = _mm256_blend_epi32(m4, m3, 0x33);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpackhi_epi64(m5, m3);
    t1 = _mm256_shuffle_epi32(m1, _mm_shuffle!(1, 0, 3, 2));
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    undiagonalize(&mut a, &mut b, &mut c, &mut d);
    // round 8
    t0 = _mm256_unpackhi_epi64(m6, m3);
    t1 = _mm256_blend_epi32(m1, m6, 0x33);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_alignr_epi8(m7, m5, 8);
    t1 = _mm256_unpackhi_epi64(m0, m4);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    diagonalize(&mut a, &mut b, &mut c, &mut d);
    t0 = _mm256_blend_epi32(m2, m1, 0x33);
    t1 = _mm256_alignr_epi8(m4, m7, 8);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpacklo_epi64(m5, m0);
    t1 = _mm256_unpacklo_epi64(m2, m3);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    undiagonalize(&mut a, &mut b, &mut c, &mut d);
    // round 9
    t0 = _mm256_unpacklo_epi64(m3, m7);
    t1 = _mm256_alignr_epi8(m0, m5, 8);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpackhi_epi64(m7, m4);
    t1 = _mm256_alignr_epi8(m4, m1, 8);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    diagonalize(&mut a, &mut b, &mut c, &mut d);
    t0 = _mm256_unpacklo_epi64(m5, m6);
    t1 = _mm256_unpackhi_epi64(m6, m0);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_alignr_epi8(m1, m2, 8);
    t1 = _mm256_alignr_epi8(m2, m3, 8);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    undiagonalize(&mut a, &mut b, &mut c, &mut d);
    // round 10
    t0 = _mm256_unpacklo_epi64(m5, m4);
    t1 = _mm256_unpackhi_epi64(m3, m0);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpacklo_epi64(m1, m2);
    t1 = _mm256_blend_epi32(m2, m3, 0x33);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    diagonalize(&mut a, &mut b, &mut c, &mut d);
    t0 = _mm256_unpackhi_epi64(m6, m7);
    t1 = _mm256_unpackhi_epi64(m4, m1);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_blend_epi32(m5, m0, 0x33);
    t1 = _mm256_unpacklo_epi64(m7, m6);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    undiagonalize(&mut a, &mut b, &mut c, &mut d);
    // round 11
    t0 = _mm256_unpacklo_epi64(m0, m1);
    t1 = _mm256_unpacklo_epi64(m2, m3);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpackhi_epi64(m0, m1);
    t1 = _mm256_unpackhi_epi64(m2, m3);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    diagonalize(&mut a, &mut b, &mut c, &mut d);
    t0 = _mm256_unpacklo_epi64(m7, m4);
    t1 = _mm256_unpacklo_epi64(m5, m6);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpackhi_epi64(m7, m4);
    t1 = _mm256_unpackhi_epi64(m5, m6);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    undiagonalize(&mut a, &mut b, &mut c, &mut d);
    // round 12
    t0 = _mm256_unpacklo_epi64(m7, m2);
    t1 = _mm256_unpackhi_epi64(m4, m6);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_unpacklo_epi64(m5, m4);
    t1 = _mm256_alignr_epi8(m3, m7, 8);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    diagonalize(&mut a, &mut b, &mut c, &mut d);
    t0 = _mm256_unpackhi_epi64(m2, m0);
    t1 = _mm256_blend_epi32(m5, m0, 0x33);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g1(&mut a, &mut b, &mut c, &mut d, &mut b0);
    t0 = _mm256_alignr_epi8(m6, m1, 8);
    t1 = _mm256_blend_epi32(m3, m1, 0x33);
    b0 = _mm256_blend_epi32(t0, t1, 0xF0);
    g2(&mut a, &mut b, &mut c, &mut d, &mut b0);
    undiagonalize(&mut a, &mut b, &mut c, &mut d);
    a = xor(a, c);
    b = xor(b, d);
    a = xor(a, iv0);
    b = xor(b, iv1);
    storeu(a, words_low);
    storeu(b, words_high);
}

#[target_feature(enable = "avx2")]
pub unsafe fn compress1_loop(
    input: &[u8],
    words: &mut [u64; 8],
    mut count: u128,
    last_node: bool,
    finalize: bool,
) {
    input_assertions(input, finalize);
    let mut local_words = *words;
    let mut fin_offset = input.len().saturating_sub(1);
    fin_offset -= fin_offset % BLOCK_BYTES;
    let mut buf = [0; BLOCK_BYTES];
    let (fin_block, fin_len, _) = final_block(input, fin_offset, &mut buf);
    let fin_last_block = flag_word(finalize);
    let fin_last_node = flag_word(finalize && last_node);
    let mut offset = 0;
    #[allow(clippy::ptr_as_ptr)]
    loop {
        let (block, count_delta, last_block, last_node) = if offset == fin_offset {
            (fin_block, fin_len, fin_last_block, fin_last_node)
        } else {
            (
                // this unsafe cast avoids bounds checks. there's guaranteed to be enough input
                // because offset < fin_offset
                &*(input.as_ptr().add(offset) as *const [u8; BLOCK_BYTES]),
                BLOCK_BYTES,
                flag_word(false),
                flag_word(false),
            )
        };
        count = count.wrapping_add(count_delta as u128);
        compress_block(block, &mut local_words, count, last_block, last_node);
        // check for termination before bumping the offset to avoid overflow.
        if offset == fin_offset {
            break;
        }
        offset += BLOCK_BYTES;
    }
    *words = local_words;
}
