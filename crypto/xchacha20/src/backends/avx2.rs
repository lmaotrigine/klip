use crate::Context;
#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm256_add_epi32, _mm256_broadcastsi128_si256, _mm256_extract_epi32,
    _mm256_set_epi32, _mm256_set_epi64x, _mm256_setzero_si256, _mm256_shuffle_epi32,
    _mm256_shuffle_epi8, _mm256_slli_epi32, _mm256_srli_epi32, _mm256_xor_si256, _mm_loadu_si128,
    _mm_storeu_si128,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, __m256i, _mm256_add_epi32, _mm256_broadcastsi128_si256, _mm256_extract_epi32,
    _mm256_set_epi32, _mm256_set_epi64x, _mm256_setzero_si256, _mm256_shuffle_epi32,
    _mm256_shuffle_epi8, _mm256_slli_epi32, _mm256_srli_epi32, _mm256_xor_si256, _mm_loadu_si128,
    _mm_storeu_si128,
};

const PAR_BLOCKS: usize = 4;
const N: usize = PAR_BLOCKS / 2;

#[inline]
#[target_feature(enable = "avx2")]
#[allow(clippy::cast_ptr_alignment, clippy::cast_sign_loss)]
pub unsafe fn inner(state: &mut [u32; 16], cx: impl Context) {
    let state_ptr = state.as_ptr().cast::<__m128i>();
    let v = [
        _mm256_broadcastsi128_si256(_mm_loadu_si128(state_ptr.add(0))),
        _mm256_broadcastsi128_si256(_mm_loadu_si128(state_ptr.add(1))),
        _mm256_broadcastsi128_si256(_mm_loadu_si128(state_ptr.add(2))),
    ];
    let mut c = _mm256_broadcastsi128_si256(_mm_loadu_si128(state_ptr.add(3)));
    c = _mm256_add_epi32(c, _mm256_set_epi32(0, 0, 0, 1, 0, 0, 0, 0));
    let mut ctr = [c; N];
    for i in &mut ctr {
        *i = c;
        c = _mm256_add_epi32(c, _mm256_set_epi32(0, 0, 0, 2, 0, 0, 0, 2));
    }
    let mut backend = Backend { v, ctr };
    cx.call(&mut backend);
    state[12] = _mm256_extract_epi32(backend.ctr[0], 0) as u32;
}

struct Backend {
    v: [__m256i; 3],
    ctr: [__m256i; N],
}

impl super::Backend<PAR_BLOCKS> for Backend {
    #[inline(always)]
    #[allow(clippy::cast_ptr_alignment)]
    fn gen_ks_block(&mut self, block: &mut crate::Block) {
        unsafe {
            let res = rounds(&self.v, &self.ctr);
            for c in &mut self.ctr {
                *c = _mm256_add_epi32(*c, _mm256_set_epi32(0, 0, 0, 1, 0, 0, 0, 1));
            }
            let res0 = core::mem::transmute::<_, [__m128i; 8]>(res[0]);
            let block_ptr = block.as_mut_ptr().cast::<__m128i>();
            for i in 0..4 {
                _mm_storeu_si128(block_ptr.add(i), res0[2 * i]);
            }
        }
    }

    #[inline(always)]
    #[allow(clippy::cast_ptr_alignment)]
    fn gen_par_ks_blocks(&mut self, blocks: &mut [crate::Block; PAR_BLOCKS]) {
        unsafe {
            let vs = rounds(&self.v, &self.ctr);
            let pb = 4;
            for c in &mut self.ctr {
                *c = _mm256_add_epi32(*c, _mm256_set_epi32(0, 0, 0, pb, 0, 0, 0, pb));
            }
            let mut block_ptr = blocks.as_mut_ptr().cast::<__m128i>();
            for v in vs {
                let t = core::mem::transmute::<_, [__m128i; 8]>(v);
                for i in 0..4 {
                    _mm_storeu_si128(block_ptr.add(i), t[2 * i]);
                    _mm_storeu_si128(block_ptr.add(4 + i), t[2 * i + 1]);
                }
                block_ptr = block_ptr.add(8);
            }
        }
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn rounds(v: &[__m256i; 3], c: &[__m256i; N]) -> [[__m256i; 4]; N] {
    let mut vs = [[_mm256_setzero_si256(); 4]; N];
    for i in 0..N {
        vs[i] = [v[0], v[1], v[2], c[i]];
    }
    for _ in 0..crate::ROUNDS {
        double_quarter_round(&mut vs);
    }
    for i in 0..N {
        for (j, vj) in v.iter().enumerate() {
            vs[i][j] = _mm256_add_epi32(vs[i][j], *vj);
        }
        vs[i][3] = _mm256_add_epi32(vs[i][3], c[i]);
    }
    vs
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn double_quarter_round(v: &mut [[__m256i; 4]; N]) {
    add_xor_rot(v);
    rows_to_cols(v);
    add_xor_rot(v);
    cols_to_rows(v);
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn rows_to_cols(vs: &mut [[__m256i; 4]; N]) {
    for [a, _, c, d] in vs {
        *c = _mm256_shuffle_epi32(*c, 0b00_11_10_01);
        *d = _mm256_shuffle_epi32(*d, 0b01_00_11_10);
        *a = _mm256_shuffle_epi32(*a, 0b10_01_00_11);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn cols_to_rows(vs: &mut [[__m256i; 4]; N]) {
    for [a, _, c, d] in vs {
        *c = _mm256_shuffle_epi32(*c, 0b10_01_00_11);
        *d = _mm256_shuffle_epi32(*d, 0b01_00_11_10);
        *a = _mm256_shuffle_epi32(*a, 0b00_11_10_01);
    }
}

#[inline]
#[target_feature(enable = "avx2")]
unsafe fn add_xor_rot(vs: &mut [[__m256i; 4]; N]) {
    let rol16_mask = _mm256_set_epi64x(
        0x0d0c_0f0e_0908_0b0a,
        0x0504_0706_0100_0302,
        0x0d0c_0f0e_0908_0b0a,
        0x0504_0706_0100_0302,
    );
    let rol8_mask = _mm256_set_epi64x(
        0x0e0d_0c0f_0a09_080b,
        0x0605_0407_0201_0003,
        0x0e0d_0c0f_0a09_080b,
        0x0605_0407_0201_0003,
    );
    for [a, b, _, d] in vs.iter_mut() {
        *a = _mm256_add_epi32(*a, *b);
        *d = _mm256_xor_si256(*d, *a);
        *d = _mm256_shuffle_epi8(*d, rol16_mask);
    }
    for [_, b, c, d] in vs.iter_mut() {
        *c = _mm256_add_epi32(*c, *d);
        *b = _mm256_xor_si256(*b, *c);
        *b = _mm256_xor_si256(_mm256_slli_epi32(*b, 12), _mm256_srli_epi32(*b, 20));
    }
    for [a, b, _, d] in vs.iter_mut() {
        *a = _mm256_add_epi32(*a, *b);
        *d = _mm256_xor_si256(*d, *a);
        *d = _mm256_shuffle_epi8(*d, rol8_mask);
    }
    for [_, b, c, d] in vs.iter_mut() {
        *c = _mm256_add_epi32(*c, *d);
        *b = _mm256_xor_si256(*b, *c);
        *b = _mm256_xor_si256(_mm256_slli_epi32(*b, 7), _mm256_srli_epi32(*b, 25));
    }
}
