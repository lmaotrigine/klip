use crate::Block;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod avx2;
pub mod soft;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod sse2;

pub trait Backend<const P: usize> {
    fn gen_ks_block(&mut self, block: &mut Block);

    #[inline(always)]
    fn gen_par_ks_blocks(&mut self, blocks: &mut [Block; P]) {
        for block in blocks {
            self.gen_ks_block(block);
        }
    }

    #[inline(always)]
    fn gen_tail_blocks(&mut self, blocks: &mut [Block]) {
        debug_assert!(blocks.len() < P);
        for block in blocks {
            self.gen_ks_block(block);
        }
    }
}
