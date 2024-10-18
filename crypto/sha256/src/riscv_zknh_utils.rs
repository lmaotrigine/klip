#[cfg(sha256_backend = "riscv-zknh")]
pub fn opaque_load<const R: usize>(k: &[u32]) -> u32 {
    assert!(R < k.len());
    let dst;
    #[cfg(target_arch = "riscv64")]
    unsafe {
        core::arch::asm!(
            "lwu {dst}, 4*{R}({k})",
            R = const R,
            k = in(reg) k.as_ptr(),
            dst = out(reg) dst,
            options(pure, readonly, nostack, preserves_flags),
        );
    }
    #[cfg(target_arch = "riscv32")]
    unsafe {
        core::arch::asm!(
            "lw {dst}, 4*{R}({k})",
            R = const R,
            k = in(reg) k.as_ptr(),
            dst = out(reg) dst,
            options(pure, readonly, nostack, preserves_flags),
        );
    }
    dst
}

#[inline(always)]
#[allow(clippy::cast_ptr_alignment, clippy::needless_range_loop)]
fn load_aligned_block(block: &[u8; 64]) -> [u32; 16] {
    let p = block.as_ptr().cast::<u32>();
    debug_assert!(p.is_aligned());
    let mut res = [0; 16];
    for i in 0..16 {
        let val = unsafe { core::ptr::read(p.add(i)) };
        res[i] = val.to_be();
    }
    res
}

#[inline(always)]
#[allow(clippy::cast_ptr_alignment, clippy::needless_range_loop)]
fn load_unaligned_block(block: &[u8; 64]) -> [u32; 16] {
    #[cfg(target_arch = "riscv32")]
    macro_rules! lw {
        ($r:literal) => {
            concat!("lw ", $r)
        };
    }
    #[cfg(target_arch = "riscv64")]
    macro_rules! lw {
        ($r:literal) => {
            concat!("lwu ", $r)
        };
    }
    let offset = (block.as_ptr() as usize) % core::mem::align_of::<u32>();
    debug_assert_ne!(offset, 0);
    let off1 = (8 * offset) % 32;
    let off2 = (32 - off1) % 32;
    let bp = block.as_ptr().wrapping_sub(offset).cast::<u32>();
    let mut left: u32;
    let mut res = [0; 16];
    unsafe {
        core::arch::asm!(
            lw!("{left}, 0({bp})"),
            "srl {left}, {left}, {off1}",
            bp = in(reg) bp,
            off1 = in(reg) off1,
            left = out(reg) left,
            options(pure, nostack, readonly, preserves_flags),
        );
    }
    for i in 0..15 {
        let right = unsafe { core::ptr::read(bp.add(1 + i)) };
        res[i] = (left | (right << off2)).to_be();
        left = right >> off1;
    }
    let right: u32;
    unsafe {
        core::arch::asm!(
            lw!("{right}, 16 * 4({bp})"),
            "sll {right}, {right}, {off2}",
            bp = in(reg) bp,
            off2 = in(reg) off2,
            right = out(reg) right,
            options(pure, nostack, readonly, preserves_flags),
        );
    }
    res[15] = (left | right).to_be();
    res
}

#[inline(always)]
#[allow(clippy::cast_ptr_alignment)]
pub fn load_block(block: &[u8; 64]) -> [u32; 16] {
    if block.as_ptr().cast::<u32>().is_aligned() {
        load_aligned_block(block)
    } else {
        load_unaligned_block(block)
    }
}
