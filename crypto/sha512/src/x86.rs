use crate::consts::K;
#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm256_add_epi64, _mm256_alignr_epi8, _mm256_extracti128_si256,
    _mm256_insertf128_si256, _mm256_set_epi64x, _mm256_set_m128i, _mm256_setzero_si256,
    _mm256_shuffle_epi8, _mm256_slli_epi64, _mm256_srli_epi64, _mm256_xor_si256, _mm_add_epi64,
    _mm_alignr_epi8, _mm_loadu_si128, _mm_setr_epi32, _mm_setzero_si128, _mm_shuffle_epi8,
    _mm_slli_epi64, _mm_srli_epi64, _mm_xor_si128,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, __m256i, _mm256_add_epi64, _mm256_alignr_epi8, _mm256_extracti128_si256,
    _mm256_insertf128_si256, _mm256_set_epi64x, _mm256_set_m128i, _mm256_setzero_si256,
    _mm256_shuffle_epi8, _mm256_slli_epi64, _mm256_srli_epi64, _mm256_xor_si256, _mm_add_epi64,
    _mm_alignr_epi8, _mm_loadu_si128, _mm_setr_epi32, _mm_setzero_si128, _mm_shuffle_epi8,
    _mm_slli_epi64, _mm_srli_epi64, _mm_xor_si128,
};

const SHA512_BLOCK_BYTE_LEN: usize = 128;
const SHA512_ROUNDS_NUM: usize = 80;
const SHA512_HASH_BYTE_LEN: usize = 64;
const SHA512_HASH_WORDS_NUM: usize = SHA512_HASH_BYTE_LEN / core::mem::size_of::<u64>();
const SHA512_BLOCK_WORDS_NUM: usize = SHA512_BLOCK_BYTE_LEN / core::mem::size_of::<u64>();
type State = [u64; SHA512_HASH_WORDS_NUM];
type MsgSchedule = [__m128i; SHA512_BLOCK_WORDS_NUM / 2];
type RoundStates = [__m128i; SHA512_ROUNDS_NUM / 2];

target_features::detect!(sha3_hwcap, "avx2");

pub fn compress(state: &mut [u64; 8], blocks: &[[u8; 128]]) {
    if sha3_hwcap::get() {
        unsafe { sha512_compress_x86_64_avx2(state, blocks) };
    } else {
        super::soft::compress(state, blocks);
    }
}

#[target_feature(enable = "avx2")]
unsafe fn sha512_compress_x86_64_avx2(state: &mut [u64; 8], blocks: &[[u8; 128]]) {
    let mut start_block = 0;
    if blocks.len() & 0b1 != 0 {
        sha512_compress_x86_64_avx(state, &blocks[0]);
        start_block += 1;
    }
    let mut ms = [_mm_setzero_si128(); 8];
    let mut t2 = [_mm_setzero_si128(); 40];
    let mut x = [_mm256_setzero_si256(); 8];
    for i in (start_block..blocks.len()).step_by(2) {
        load_data_avx2(&mut x, &mut ms, &mut t2, blocks.as_ptr().add(i).cast());
        let mut current_state = *state;
        rounds_0_63_avx2(&mut current_state, &mut x, &mut ms, &mut t2);
        rounds_64_79(&mut current_state, &ms);
        accumulate_state(state, &current_state);
        current_state = *state;
        process_second_block(&mut current_state, &t2);
        accumulate_state(state, &current_state);
    }
}

#[inline(always)]
unsafe fn sha512_compress_x86_64_avx(state: &mut [u64; 8], block: &[u8; 128]) {
    let mut ms = [_mm_setzero_si128(); 8];
    let mut x = [_mm_setzero_si128(); 8];
    let mut current_state = *state;
    load_data_avx(&mut x, &mut ms, block.as_ptr().cast());
    rounds_0_63_avx(&mut current_state, &mut x, &mut ms);
    rounds_64_79(&mut current_state, &ms);
    accumulate_state(state, &current_state);
}

#[inline(always)]
unsafe fn load_data_avx(x: &mut [__m128i; 8], ms: &mut MsgSchedule, data: *const __m128i) {
    let mask = _mm_setr_epi32(0x0405_0607, 0x0001_0203, 0x0c0d_0e0f, 0x0809_0a0b);
    macro_rules! unrolled_iterations {
        ($($i:literal),*$(,)?) => {
            $(
                x[$i] = _mm_loadu_si128(data.add($i).cast());
                x[$i] = _mm_shuffle_epi8(x[$i], mask);
                let k: *const u64 = &K[2 * $i];
                let y = _mm_add_epi64(x[$i], _mm_loadu_si128(k.cast()));
                ms[$i] = y;
            )*
        };
    }
    unrolled_iterations!(0, 1, 2, 3, 4, 5, 6, 7);
}

#[inline(always)]
unsafe fn load_data_avx2(
    x: &mut [__m256i; 8],
    ms: &mut MsgSchedule,
    t2: &mut RoundStates,
    data: *const __m128i,
) {
    let mask = _mm256_set_epi64x(
        0x0809_0a0b_0c0d_0e0f,
        0x0001_0203_0405_0607,
        0x0809_0a0b_0c0d_0e0f,
        0x0001_0203_0405_0607,
    );
    macro_rules! unrolled_iterations {
        ($($i:literal),*$(,)?) => {
            $(
                x[$i] = _mm256_insertf128_si256(x[$i], _mm_loadu_si128(data.add(8 + $i).cast()), 1);
                x[$i] = _mm256_insertf128_si256(x[$i], _mm_loadu_si128(data.add($i).cast()), 0);
                x[$i] = _mm256_shuffle_epi8(x[$i], mask);
                let t = _mm_loadu_si128(K.as_ptr().add($i * 2).cast());
                let y = _mm256_add_epi64(x[$i], _mm256_set_m128i(t, t));
                ms[$i] = _mm256_extracti128_si256(y, 0);
                t2[$i] = _mm256_extracti128_si256(y, 1);
            )*
        };
    }
    unrolled_iterations!(0, 1, 2, 3, 4, 5, 6, 7);
}

#[inline(always)]
unsafe fn rounds_0_63_avx(current_state: &mut State, x: &mut [__m128i; 8], ms: &mut MsgSchedule) {
    let mut k_idx = SHA512_BLOCK_WORDS_NUM;
    for _ in 0..4 {
        for j in 0..8 {
            let k: *const u64 = &K[k_idx];
            let k = _mm_loadu_si128(k.cast());
            let y = sha512_update_x_avx(x, k);
            {
                let ms = cast_ms(ms);
                sha_round(current_state, ms[2 * j]);
                sha_round(current_state, ms[2 * j + 1]);
            }
            ms[j] = y;
            k_idx += 2;
        }
    }
}

#[inline(always)]
unsafe fn rounds_0_63_avx2(
    current_state: &mut State,
    x: &mut [__m256i; 8],
    ms: &mut MsgSchedule,
    t2: &mut RoundStates,
) {
    let mut kx4_idx = SHA512_BLOCK_WORDS_NUM;
    for i in 1..5 {
        for j in 0..8 {
            let t = _mm_loadu_si128(K.as_ptr().add(kx4_idx).cast());
            let y = sha512_update_x_avx2(x, _mm256_set_m128i(t, t));
            {
                let ms = cast_ms(ms);
                sha_round(current_state, ms[2 * j]);
                sha_round(current_state, ms[2 * j + 1]);
            }
            ms[j] = _mm256_extracti128_si256(y, 0);
            t2[8 * i + j] = _mm256_extracti128_si256(y, 1);
            kx4_idx += 2;
        }
    }
}

#[inline(always)]
fn rounds_64_79(current_state: &mut State, ms: &MsgSchedule) {
    let ms = cast_ms(ms);
    for i in 64..80 {
        sha_round(current_state, ms[i & 0xf]);
    }
}

#[inline(always)]
fn process_second_block(current_state: &mut State, t2: &RoundStates) {
    for t2 in cast_rs(t2) {
        sha_round(current_state, *t2);
    }
}

#[inline(always)]
fn sha_round(s: &mut State, x: u64) {
    macro_rules! big_sigma0 {
        ($a:expr) => {
            $a.rotate_right(28) ^ $a.rotate_right(34) ^ $a.rotate_right(39)
        };
    }
    macro_rules! big_sigma1 {
        ($a:expr) => {
            $a.rotate_right(14) ^ $a.rotate_right(18) ^ $a.rotate_right(41)
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
    macro_rules! rotate_state {
        ($s:ident) => {{
            let tmp = $s[7];
            $s[7] = $s[6];
            $s[6] = $s[5];
            $s[5] = $s[4];
            $s[4] = $s[3];
            $s[3] = $s[2];
            $s[2] = $s[1];
            $s[1] = $s[0];
            $s[0] = tmp;
        }};
    }
    let t = x
        .wrapping_add(s[7])
        .wrapping_add(big_sigma1!(s[4]))
        .wrapping_add(bool3ary_202!(s[4], s[5], s[6]));
    s[7] = t
        .wrapping_add(big_sigma0!(s[0]))
        .wrapping_add(bool3ary_232!(s[0], s[1], s[2]));
    s[3] = s[3].wrapping_add(t);
    rotate_state!(s);
}

#[inline(always)]
fn accumulate_state(dst: &mut State, src: &State) {
    for i in 0..SHA512_HASH_WORDS_NUM {
        dst[i] = dst[i].wrapping_add(src[i]);
    }
}

macro_rules! fn_sha512_update_x {
    ($name:ident, $ty:ty, {add64 = $add64:ident, alignr8 = $alignr8:ident, srl64 = $srl64:ident, sll64 = $sll64:ident, xor = $xor:ident}) => {
        unsafe fn $name(x: &mut [$ty; 8], k: $ty) -> $ty {
            let mut t0 = $alignr8(x[1], x[0], 8);
            let mut t3 = $alignr8(x[5], x[4], 8);
            let mut t2 = $srl64(t0, 1);
            x[0] = $add64(x[0], t3);
            t3 = $srl64(t0, 7);
            let mut t1 = $sll64(t0, 56);
            t0 = $xor(t3, t2);
            t2 = $srl64(t2, 7);
            t0 = $xor(t0, t1);
            t1 = $sll64(t1, 7);
            t0 = $xor(t0, t2);
            t0 = $xor(t0, t1);
            t3 = $srl64(x[7], 6);
            t2 = $sll64(x[7], 3);
            x[0] = $add64(x[0], t0);
            t1 = $srl64(x[7], 19);
            t3 = $xor(t3, t2);
            t2 = $sll64(t2, 42);
            t3 = $xor(t3, t1);
            t1 = $srl64(t1, 42);
            t3 = $xor(t3, t2);
            t3 = $xor(t3, t1);
            x[0] = $add64(x[0], t3);
            let temp = x[0];
            x[0] = x[1];
            x[1] = x[2];
            x[2] = x[3];
            x[3] = x[4];
            x[4] = x[5];
            x[5] = x[6];
            x[6] = x[7];
            x[7] = temp;
            $add64(x[7], k)
        }
    };
}

fn_sha512_update_x!(sha512_update_x_avx, __m128i, { add64 = _mm_add_epi64, alignr8 = _mm_alignr_epi8, srl64 = _mm_srli_epi64, sll64 = _mm_slli_epi64, xor = _mm_xor_si128});
fn_sha512_update_x!(sha512_update_x_avx2, __m256i, { add64 = _mm256_add_epi64, alignr8 = _mm256_alignr_epi8, srl64 = _mm256_srli_epi64, sll64 = _mm256_slli_epi64, xor = _mm256_xor_si256});

#[inline(always)]
const fn cast_ms(ms: &MsgSchedule) -> &[u64; SHA512_BLOCK_WORDS_NUM] {
    unsafe { &*(ms.as_ptr().cast()) }
}

#[inline(always)]
const fn cast_rs(rs: &RoundStates) -> &[u64; SHA512_ROUNDS_NUM] {
    unsafe { &*(rs.as_ptr().cast()) }
}
