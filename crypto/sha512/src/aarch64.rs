use crate::consts::K;
use core::arch::aarch64::{
    vaddq_u64, vextq_u64, vld1q_u64, vld1q_u8, vreinterpretq_u64_u8, vrev64q_u8, vsha512h2q_u64,
    vsha512hq_u64, vsha512su0q_u64, vsha512su1q_u64, vst1q_u64,
};

target_features::detect!(sha3_hwcap, "sha3");

pub fn compress(state: &mut [u64; 8], blocks: &[[u8; 128]]) {
    if sha3_hwcap::get() {
        unsafe { sha512_compress(state, blocks) };
    } else {
        super::soft::compress(state, blocks);
    }
}

#[target_feature(enable = "sha3")]
#[allow(clippy::too_many_lines)]
unsafe fn sha512_compress(state: &mut [u64; 8], blocks: &[[u8; 128]]) {
    let mut ab = vld1q_u64(state[0..2].as_ptr());
    let mut cd = vld1q_u64(state[2..4].as_ptr());
    let mut ef = vld1q_u64(state[4..6].as_ptr());
    let mut gh = vld1q_u64(state[6..8].as_ptr());
    for block in blocks {
        let ab_orig = ab;
        let cd_orig = cd;
        let ef_orig = ef;
        let gh_orig = gh;
        let mut s0 = vreinterpretq_u64_u8(vrev64q_u8(vld1q_u8(block[0..16].as_ptr())));
        let mut s1 = vreinterpretq_u64_u8(vrev64q_u8(vld1q_u8(block[16..32].as_ptr())));
        let mut s2 = vreinterpretq_u64_u8(vrev64q_u8(vld1q_u8(block[32..48].as_ptr())));
        let mut s3 = vreinterpretq_u64_u8(vrev64q_u8(vld1q_u8(block[48..64].as_ptr())));
        let mut s4 = vreinterpretq_u64_u8(vrev64q_u8(vld1q_u8(block[64..80].as_ptr())));
        let mut s5 = vreinterpretq_u64_u8(vrev64q_u8(vld1q_u8(block[80..96].as_ptr())));
        let mut s6 = vreinterpretq_u64_u8(vrev64q_u8(vld1q_u8(block[96..112].as_ptr())));
        let mut s7 = vreinterpretq_u64_u8(vrev64q_u8(vld1q_u8(block[112..128].as_ptr())));
        let mut initial_sum = vaddq_u64(s0, vld1q_u64(&K[0]));
        let mut sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), gh);
        let mut intermediate = vsha512hq_u64(sum, vextq_u64(ef, gh, 1), vextq_u64(cd, ef, 1));
        gh = vsha512h2q_u64(intermediate, cd, ab);
        cd = vaddq_u64(cd, intermediate);
        initial_sum = vaddq_u64(s2, vld1q_u64(&K[4]));
        sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), cd);
        intermediate = vsha512hq_u64(sum, vextq_u64(ab, cd, 1), vextq_u64(gh, ab, 1));
        cd = vsha512h2q_u64(intermediate, gh, ef);
        gh = vaddq_u64(gh, intermediate);
        initial_sum = vaddq_u64(s3, vld1q_u64(&K[6]));
        sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), ab);
        intermediate = vsha512hq_u64(sum, vextq_u64(gh, ab, 1), vextq_u64(ef, gh, 1));
        ab = vsha512h2q_u64(intermediate, ef, cd);
        ef = vaddq_u64(ef, intermediate);
        initial_sum = vaddq_u64(s4, vld1q_u64(&K[8]));
        sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), gh);
        intermediate = vsha512hq_u64(sum, vextq_u64(ef, gh, 1), vextq_u64(cd, ef, 1));
        gh = vsha512h2q_u64(intermediate, cd, ab);
        cd = vaddq_u64(cd, intermediate);
        initial_sum = vaddq_u64(s5, vld1q_u64(&K[10]));
        sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), ef);
        intermediate = vsha512hq_u64(sum, vextq_u64(cd, ef, 1), vextq_u64(ab, cd, 1));
        ef = vsha512h2q_u64(intermediate, ab, gh);
        ab = vaddq_u64(ab, intermediate);
        initial_sum = vaddq_u64(s6, vld1q_u64(&K[12]));
        sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), cd);
        intermediate = vsha512hq_u64(sum, vextq_u64(ab, cd, 1), vextq_u64(gh, ab, 1));
        cd = vsha512h2q_u64(intermediate, gh, ef);
        gh = vaddq_u64(gh, intermediate);
        initial_sum = vaddq_u64(s7, vld1q_u64(&K[14]));
        sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), ab);
        intermediate = vsha512hq_u64(sum, vextq_u64(gh, ab, 1), vextq_u64(ef, gh, 1));
        ab = vsha512h2q_u64(intermediate, ef, cd);
        ef = vaddq_u64(ef, intermediate);
        for t in (16..80).step_by(16) {
            s0 = vsha512su1q_u64(vsha512su0q_u64(s0, s1), s7, vextq_u64(s4, s5, 1));
            initial_sum = vaddq_u64(s0, vld1q_u64(&K[t]));
            sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), gh);
            intermediate = vsha512hq_u64(sum, vextq_u64(ef, gh, 1), vextq_u64(cd, ef, 1));
            gh = vsha512h2q_u64(intermediate, cd, ab);
            cd = vaddq_u64(cd, intermediate);
            s1 = vsha512su1q_u64(vsha512su0q_u64(s1, s2), s0, vextq_u64(s5, s6, 1));
            initial_sum = vaddq_u64(s1, vld1q_u64(&K[t + 2]));
            sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), ef);
            intermediate = vsha512hq_u64(sum, vextq_u64(cd, ef, 1), vextq_u64(ab, cd, 1));
            ef = vsha512h2q_u64(intermediate, ab, gh);
            ab = vaddq_u64(ab, intermediate);
            s2 = vsha512su1q_u64(vsha512su0q_u64(s2, s3), s1, vextq_u64(s6, s7, 1));
            initial_sum = vaddq_u64(s2, vld1q_u64(&K[t + 4]));
            sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), cd);
            intermediate = vsha512hq_u64(sum, vextq_u64(ab, cd, 1), vextq_u64(gh, ab, 1));
            cd = vsha512h2q_u64(intermediate, gh, ef);
            gh = vaddq_u64(gh, intermediate);
            s3 = vsha512su1q_u64(vsha512su0q_u64(s3, s4), s2, vextq_u64(s7, s0, 1));
            initial_sum = vaddq_u64(s3, vld1q_u64(&K[t + 6]));
            sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), ab);
            intermediate = vsha512hq_u64(sum, vextq_u64(gh, ab, 1), vextq_u64(ef, gh, 1));
            ab = vsha512h2q_u64(intermediate, ef, cd);
            ef = vaddq_u64(ef, intermediate);
            s4 = vsha512su1q_u64(vsha512su0q_u64(s4, s5), s3, vextq_u64(s0, s1, 1));
            initial_sum = vaddq_u64(s4, vld1q_u64(&K[t + 8]));
            sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), gh);
            intermediate = vsha512hq_u64(sum, vextq_u64(ef, gh, 1), vextq_u64(cd, ef, 1));
            gh = vsha512h2q_u64(intermediate, cd, ab);
            cd = vaddq_u64(cd, intermediate);
            s5 = vsha512su1q_u64(vsha512su0q_u64(s5, s6), s4, vextq_u64(s1, s2, 1));
            initial_sum = vaddq_u64(s5, vld1q_u64(&K[t + 10]));
            sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), ef);
            intermediate = vsha512hq_u64(sum, vextq_u64(cd, ef, 1), vextq_u64(ab, cd, 1));
            ef = vsha512h2q_u64(intermediate, ab, gh);
            ab = vaddq_u64(ab, intermediate);
            s6 = vsha512su1q_u64(vsha512su0q_u64(s6, s7), s5, vextq_u64(s2, s3, 1));
            initial_sum = vaddq_u64(s6, vld1q_u64(&K[t + 12]));
            sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), cd);
            intermediate = vsha512hq_u64(sum, vextq_u64(ab, cd, 1), vextq_u64(gh, ab, 1));
            cd = vsha512h2q_u64(intermediate, gh, ef);
            gh = vaddq_u64(gh, intermediate);
            s7 = vsha512su1q_u64(vsha512su0q_u64(s7, s0), s6, vextq_u64(s3, s4, 1));
            initial_sum = vaddq_u64(s7, vld1q_u64(&K[t + 14]));
            sum = vaddq_u64(vextq_u64(initial_sum, initial_sum, 1), ab);
            intermediate = vsha512hq_u64(sum, vextq_u64(gh, ab, 1), vextq_u64(ef, gh, 1));
            ab = vsha512h2q_u64(intermediate, ef, cd);
            ef = vaddq_u64(ef, intermediate);
        }
        ab = vaddq_u64(ab, ab_orig);
        cd = vaddq_u64(cd, cd_orig);
        ef = vaddq_u64(ef, ef_orig);
        gh = vaddq_u64(gh, gh_orig);
    }
    vst1q_u64(state[0..2].as_mut_ptr(), ab);
    vst1q_u64(state[2..4].as_mut_ptr(), cd);
    vst1q_u64(state[4..6].as_mut_ptr(), ef);
    vst1q_u64(state[6..8].as_mut_ptr(), gh);
}
