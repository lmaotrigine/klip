use crate::Context;
use core::arch::aarch64::{
    uint32x4_t, vaddq_u32, veorq_u32, vextq_u32, vld1q_u32, vld1q_u8, vorrq_u32, vqtbl1q_u8,
    vreinterpretq_u16_u32, vreinterpretq_u32_u16, vreinterpretq_u32_u8, vreinterpretq_u8_u32,
    vrev32q_u16, vshlq_n_u32, vshrq_n_u32, vst1q_u32, vst1q_u8,
};

macro_rules! rotate_left {
    ($v:expr, 8) => {{
        let maskb = [3, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10, 15, 12, 13, 14];
        let mask = vld1q_u8(maskb.as_ptr());
        $v = vreinterpretq_u32_u8(vqtbl1q_u8(vreinterpretq_u8_u32($v), mask));
    }};
    ($v:expr, 16) => {
        $v = vreinterpretq_u32_u16(vrev32q_u16(vreinterpretq_u16_u32($v)))
    };
    ($v:expr, $r:literal) => {
        $v = vorrq_u32(vshlq_n_u32($v, $r), vshrq_n_u32($v, 32 - $r))
    };
}

macro_rules! extract {
    ($v:expr, $s:literal) => {
        $v = vextq_u32($v, $v, $s)
    };
}

macro_rules! add_assign {
    ($a:expr, $b:expr) => {
        $a = vaddq_u32($a, $b)
    };
}

#[inline]
unsafe fn rows_to_cols(blocks: &mut [[uint32x4_t; 4]; 4]) {
    for block in blocks.iter_mut() {
        extract!(block[1], 1);
        extract!(block[2], 2);
        extract!(block[3], 3);
    }
}

#[inline]
unsafe fn cols_to_rows(blocks: &mut [[uint32x4_t; 4]; 4]) {
    for block in blocks.iter_mut() {
        extract!(block[1], 3);
        extract!(block[2], 2);
        extract!(block[3], 1);
    }
}

#[inline]
unsafe fn add_xor_rot(blocks: &mut [[uint32x4_t; 4]; 4]) {
    macro_rules! xor_assign {
        ($a:expr, $b:expr) => {
            $a = veorq_u32($a, $b)
        };
    }
    for block in blocks.iter_mut() {
        add_assign!(block[0], block[1]);
        xor_assign!(block[3], block[0]);
        rotate_left!(block[3], 16);
        add_assign!(block[2], block[3]);
        xor_assign!(block[1], block[2]);
        rotate_left!(block[1], 12);
        add_assign!(block[0], block[1]);
        xor_assign!(block[3], block[0]);
        rotate_left!(block[3], 8);
        add_assign!(block[2], block[3]);
        xor_assign!(block[1], block[2]);
        rotate_left!(block[1], 7);
    }
}

#[inline]
unsafe fn double_quarter_round(blocks: &mut [[uint32x4_t; 4]; 4]) {
    add_xor_rot(blocks);
    rows_to_cols(blocks);
    add_xor_rot(blocks);
    cols_to_rows(blocks);
}

struct Backend {
    state: [uint32x4_t; 4],
    ctrs: [uint32x4_t; 4],
}

impl Backend {
    #[inline]
    unsafe fn new(state: &mut [u32; 16]) -> Self {
        let state = [
            vld1q_u32(state.as_ptr().offset(0)),
            vld1q_u32(state.as_ptr().offset(4)),
            vld1q_u32(state.as_ptr().offset(8)),
            vld1q_u32(state.as_ptr().offset(12)),
        ];
        let ctrs = [
            vld1q_u32([1, 0, 0, 0].as_ptr()),
            vld1q_u32([2, 0, 0, 0].as_ptr()),
            vld1q_u32([3, 0, 0, 0].as_ptr()),
            vld1q_u32([4, 0, 0, 0].as_ptr()),
        ];
        Self { state, ctrs }
    }
}

impl super::Backend<4> for Backend {
    #[inline(always)]
    fn gen_ks_block(&mut self, block: &mut crate::Block) {
        let state3 = self.state[3];
        let mut par = [[0; 64]; 4];
        self.gen_par_ks_blocks(&mut par);
        *block = par[0];
        unsafe {
            self.state[3] = vaddq_u32(state3, vld1q_u32([1, 0, 0, 0].as_ptr()));
        }
    }

    #[inline(always)]
    #[allow(clippy::cast_sign_loss)]
    fn gen_par_ks_blocks(&mut self, dest: &mut [crate::Block; 4]) {
        unsafe {
            let mut blocks = [
                [self.state[0], self.state[1], self.state[2], self.state[3]],
                [
                    self.state[0],
                    self.state[1],
                    self.state[2],
                    vaddq_u32(self.state[3], self.ctrs[0]),
                ],
                [
                    self.state[0],
                    self.state[1],
                    self.state[2],
                    vaddq_u32(self.state[3], self.ctrs[1]),
                ],
                [
                    self.state[0],
                    self.state[1],
                    self.state[2],
                    vaddq_u32(self.state[3], self.ctrs[2]),
                ],
            ];
            for _ in 0..crate::ROUNDS {
                double_quarter_round(&mut blocks);
            }
            for block in 0..4 {
                for state_row in 0..4 {
                    add_assign!(blocks[block][state_row], self.state[state_row]);
                }
                if block > 0 {
                    blocks[block][3] = vaddq_u32(blocks[block][3], self.ctrs[block - 1]);
                }
                for state_row in 0..4 {
                    vst1q_u8(
                        dest[block].as_mut_ptr().offset(state_row << 4),
                        vreinterpretq_u8_u32(blocks[block][state_row as usize]),
                    );
                }
            }
            self.state[3] = vaddq_u32(self.state[3], self.ctrs[3]);
        }
    }
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn inner(state: &mut [u32; 16], ctx: impl Context) {
    let mut backend = Backend::new(state);
    ctx.call(&mut backend);
    vst1q_u32(state.as_mut_ptr().offset(12), backend.state[3]);
}
