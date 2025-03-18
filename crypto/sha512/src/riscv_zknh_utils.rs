#[cfg(sha512_backend = "riscv-zknh")]
pub fn opaque_load<const R: usize>(k: &[u64]) -> u64 {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        let dst;
        core::arch::asm!(
            "ld {dst}, {N}({k})",
            N = const 8 * R,
            k = in(reg) k.as_ptr(),
            dst = out(reg) dst,
            options(pure, readonly, nostack, preserves_flags),
        );
        dst
    }
    #[cfg(target_arch = "riscv32")]
    #[allow(clippy::cast_lossless)]
    unsafe {
        let [hi, lo]: [u32; 2];
        core::arch::asm!(
            "lw {lo}, {N1}({k})",
            "lw {hi}, {N2}({k})",
            N1 = const 8 * R,
            N2 = const 8 * R + 4,
            k = in(reg) k.as_ptr(),
            lo = out(reg) lo,
            hi = out(reg) hi,
            options(pure, readonly, nostack, preserves_flags),
        );
        ((hi as u64) << 32) | (lo as u64)
    }
}

#[cfg(target_arch = "riscv32")]
#[allow(clippy::cast_lossless, clippy::cast_ptr_alignment)]
fn load_aligned_block(block: &[u8; 128]) -> [u64; 16] {
    let p = block.as_ptr().cast::<[u32; 32]>();
    debug_assert!(p.is_aligned());
    let block = unsafe { &*p };
    let mut res = [0; 16];
    for i in 0..16 {
        let a = block[2 * i].to_be() as u64;
        let b = block[2 * i + 1].to_be() as u64;
        res[i] = (a << 32) | b;
    }
    res
}

#[cfg(target_arch = "riscv32")]
#[allow(
    clippy::cast_lossless,
    clippy::cast_ptr_alignment,
    clippy::needless_range_loop
)]
fn load_unaligned_block(block: &[u8; 128]) -> [u64; 16] {
    let offset = (block.as_ptr() as usize) % core::mem::align_of::<u32>();
    debug_assert_ne!(offset, 0);
    let off1 = (8 * offset) % 32;
    let off2 = (32 - off1) % 32;
    let bp = block.as_ptr().wrapping_sub(offset).cast::<u32>();
    let mut left: u32;
    let mut block32 = [0; 32];
    unsafe {
        core::arch::asm!(
            "lw {left}, 0({bp})",
            "srl {left}, {left}, {off1}",
            bp = in(reg) bp,
            off1 = in(reg) off1,
            left = out(reg) left,
            options(pure, readonly, nostack, preserves_flags),
        );
    }
    for i in 0..31 {
        let right = unsafe { core::ptr::read(bp.add(1 + i)) };
        block32[i] = left | (right << off2);
        left = right >> off1;
    }
    let right: u32;
    unsafe {
        core::arch::asm!(
            "lw {right}, 4({bp})",
            "sll {right}, {right}, {off2}",
            bp = in(reg) bp,
            off2 = in(reg) off2,
            right = out(reg) right,
            options(pure, readonly, nostack, preserves_flags),
        );
    }
    block32[31] = left | right;
    let mut block64 = [0; 16];
    for i in 0..16 {
        let a = block32[2 * i].to_be() as u64;
        let b = block32[2 * i + 1].to_be() as u64;
        block64[i] = (a << 32) | b;
    }
    block64
}

#[cfg(target_arch = "riscv64")]
#[allow(clippy::cast_ptr_alignment, clippy::needless_range_loop)]
fn load_aligned_block(block: &[u8; 128]) -> [u64; 16] {
    let p = block.as_ptr().cast::<u64>();
    debug_assert!(p.is_aligned());
    let mut res = [0; 16];
    for i in 0..16 {
        let val = unsafe { core::ptr::read(p.add(i)) };
        res[i] = val.to_be();
    }
    res
}

#[cfg(target_arch = "riscv64")]
#[allow(clippy::cast_ptr_alignment, clippy::needless_range_loop)]
fn load_unaligned_block(block: &[u8; 128]) -> [u64; 16] {
    let offset = (block.as_ptr() as usize) % core::mem::align_of::<u64>();
    debug_assert_ne!(offset, 0);
    let off1 = (8 * offset) % 64;
    let off2 = (64 - off1) % 64;
    let bp = block.as_ptr().wrapping_sub(offset).cast::<u64>();
    let mut left: u64;
    let mut res = [0; 16];
    unsafe {
        core::arch::asm!(
            "ld {left}, 0({bp})",
            "srl {left}, {left}, {off1}",
            bp = in(reg) bp,
            off1 = in(reg) off1,
            left = out(reg) left,
            options(pure, readonly, nostack, preserves_flags),
        );
    }
    for i in 0..15 {
        let right = unsafe { core::ptr::read(bp.add(1 + i)) };
        res[i] = (left | (right << off2)).to_be();
        left = right >> off1;
    }
    let right: u64;
    unsafe {
        core::arch::asm!(
            "ld {right}, 16 * 8({bp})",
            "sll {right}, {right}, {off2}",
            bp = in(reg) bp,
            off2 = in(reg) off2,
            right = out(reg) right,
            options(pure, readonly, nostack, preserves_flags),
        );
    }
    res[15] = (left | right).to_be();
    res
}

#[inline(always)]
#[allow(clippy::cast_ptr_alignment)]
pub fn load_block(block: &[u8; 128]) -> [u64; 16] {
    if block.as_ptr().cast::<usize>().is_aligned() {
        load_aligned_block(block)
    } else {
        load_unaligned_block(block)
    }
}
