use crypto_common::erase::{Erase, EraseOnDrop};

const STATE_WORDS: usize = 16;
const ROUNDS: usize = 4;

type Block = [u8; 64];

pub struct Salsa {
    state: [u32; STATE_WORDS],
} // SalsaCore<10>

impl Salsa {
    pub const fn from_raw_state(state: [u32; STATE_WORDS]) -> Self {
        Self { state }
    }

    #[inline(always)]
    const fn get_block_pos(&self) -> u64 {
        (self.state[8] as u64) + ((self.state[9] as u64) << 32)
    }

    #[inline(always)]
    const fn set_block_pos(&mut self, pos: u64) {
        self.state[8] = (pos & 0xffff_ffff) as u32;
        self.state[9] = ((pos >> 32) & 0xffff_ffff) as u32;
    }

    #[inline(always)]
    pub fn write_keystream_block(&mut self, block: &mut Block) {
        let res = run_rounds(&self.state);
        self.set_block_pos(self.get_block_pos() + 1);
        for (chunk, val) in block.chunks_exact_mut(4).zip(res.iter()) {
            chunk.copy_from_slice(&val.to_le_bytes());
        }
    }
}

impl Drop for Salsa {
    fn drop(&mut self) {
        self.state.erase();
    }
}

impl EraseOnDrop for Salsa {}

#[inline]
const fn quarter_round(a: usize, b: usize, c: usize, d: usize, state: &mut [u32; STATE_WORDS]) {
    state[b] ^= state[a].wrapping_add(state[d]).rotate_left(7);
    state[c] ^= state[b].wrapping_add(state[a]).rotate_left(9);
    state[d] ^= state[c].wrapping_add(state[b]).rotate_left(13);
    state[a] ^= state[d].wrapping_add(state[c]).rotate_left(18);
}

#[inline(always)]
fn run_rounds(state: &[u32; STATE_WORDS]) -> [u32; STATE_WORDS] {
    let mut res = *state;
    for _ in 0..ROUNDS {
        quarter_round(0, 4, 8, 12, &mut res);
        quarter_round(5, 9, 13, 1, &mut res);
        quarter_round(10, 14, 2, 6, &mut res);
        quarter_round(15, 3, 7, 11, &mut res);
        quarter_round(0, 1, 2, 3, &mut res);
        quarter_round(5, 6, 7, 4, &mut res);
        quarter_round(10, 11, 8, 9, &mut res);
        quarter_round(15, 12, 13, 14, &mut res);
    }
    for (s1, s0) in res.iter_mut().zip(state.iter()) {
        *s1 = s1.wrapping_add(*s0);
    }
    res
}
