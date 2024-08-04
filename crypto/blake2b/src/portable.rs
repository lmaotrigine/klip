#![allow(clippy::inline_always)]

use crate::{
    core::{count_high, count_low, final_block, flag_word, input_assertions},
    utils::{as_array, as_arrays},
    BLOCK_BYTES, IV, SIGMA,
};

#[inline(always)]
fn g(v: &mut [u64; 16], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
    v[d] = (v[d] ^ v[a]).rotate_right(32);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(24);
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
    v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(63);
}

#[cfg_attr(not(feature = "uninline"), inline(always))]
fn round(r: usize, m: &[u64; 16], v: &mut [u64; 16]) {
    // select the message schedule based on the round
    let s = SIGMA[r];
    // mix the columns.
    g(v, 0, 4, 8, 12, m[s[0] as usize], m[s[1] as usize]);
    g(v, 1, 5, 9, 13, m[s[2] as usize], m[s[3] as usize]);
    g(v, 2, 6, 10, 14, m[s[4] as usize], m[s[5] as usize]);
    g(v, 3, 7, 11, 15, m[s[6] as usize], m[s[7] as usize]);
    // mix the rows
    g(v, 0, 5, 10, 15, m[s[8] as usize], m[s[9] as usize]);
    g(v, 1, 6, 11, 12, m[s[10] as usize], m[s[11] as usize]);
    g(v, 2, 7, 8, 13, m[s[12] as usize], m[s[13] as usize]);
    g(v, 3, 4, 9, 14, m[s[14] as usize], m[s[15] as usize]);
}

#[inline(always)]
fn compress_block(
    block: &[u8; BLOCK_BYTES],
    words: &mut [u64; 8],
    count: u128,
    last_block: u64,
    last_node: u64,
) {
    const W: usize = core::mem::size_of::<u64>();
    // initialize the compression state.
    let mut v = [
        words[0],
        words[1],
        words[2],
        words[3],
        words[4],
        words[5],
        words[6],
        words[7],
        IV[0],
        IV[1],
        IV[2],
        IV[3],
        IV[4] ^ count_low(count),
        IV[5] ^ count_high(count),
        IV[6] ^ last_block,
        IV[7] ^ last_node,
    ];
    // parse the message bytes as ints in little endian order
    let msg_arrs = as_arrays!(block, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W);
    let m = [
        u64::from_le_bytes(*msg_arrs.0),
        u64::from_le_bytes(*msg_arrs.1),
        u64::from_le_bytes(*msg_arrs.2),
        u64::from_le_bytes(*msg_arrs.3),
        u64::from_le_bytes(*msg_arrs.4),
        u64::from_le_bytes(*msg_arrs.5),
        u64::from_le_bytes(*msg_arrs.6),
        u64::from_le_bytes(*msg_arrs.7),
        u64::from_le_bytes(*msg_arrs.8),
        u64::from_le_bytes(*msg_arrs.9),
        u64::from_le_bytes(*msg_arrs.10),
        u64::from_le_bytes(*msg_arrs.11),
        u64::from_le_bytes(*msg_arrs.12),
        u64::from_le_bytes(*msg_arrs.13),
        u64::from_le_bytes(*msg_arrs.14),
        u64::from_le_bytes(*msg_arrs.15),
    ];
    round(0, &m, &mut v);
    round(1, &m, &mut v);
    round(2, &m, &mut v);
    round(3, &m, &mut v);
    round(4, &m, &mut v);
    round(5, &m, &mut v);
    round(6, &m, &mut v);
    round(7, &m, &mut v);
    round(8, &m, &mut v);
    round(9, &m, &mut v);
    round(10, &m, &mut v);
    round(11, &m, &mut v);
    words[0] ^= v[0] ^ v[8];
    words[1] ^= v[1] ^ v[9];
    words[2] ^= v[2] ^ v[10];
    words[3] ^= v[3] ^ v[11];
    words[4] ^= v[4] ^ v[12];
    words[5] ^= v[5] ^ v[13];
    words[6] ^= v[6] ^ v[14];
    words[7] ^= v[7] ^ v[15];
}

pub fn compress1_loop(
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
    loop {
        let (block, count_delta, last_block, last_node) = if offset == fin_offset {
            (fin_block, fin_len, fin_last_block, fin_last_node)
        } else {
            (
                as_array!(input, offset, BLOCK_BYTES),
                BLOCK_BYTES,
                flag_word(false),
                flag_word(false),
            )
        };
        count = count.wrapping_add(count_delta as u128);
        compress_block(block, &mut local_words, count, last_block, last_node);
        // check for termination before bumping the offset, to avoid overflow.
        if offset == fin_offset {
            break;
        }
        offset += BLOCK_BYTES;
    }
    *words = local_words;
}
