//! `x86`/`x86_64` CPU feature detection support.
//!
//! Portable, `#![no_std]` friendly implementation that relies on the x86
//! `CPUID` instruction for feature detection.

#[macro_export]
#[doc(hidden)]
macro_rules! __unless {
    ($($tf:tt),+ => $body:expr) => {{
        #[cfg(not(all($(target_feature = $tf,)+)))]
        {
            #[cfg(not(any(target_env = "sgx", target_os = "", target_os = "uefi")))]
            $body
            // CPUID isn't available on SGX. Freestanding and UEFI targets don't
            // support SIMD with default compilation flags, so false it is.
            #[cfg(any(target_env = "sgx", target_os = "", target_os = "uefi"))]
            false
        }
        #[cfg(all($(target_feature = $tf,)+))]
        true
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __detect {
    ($($tf:tt),+) => {{
        #[cfg(target_arch = "x86")]
        use core::arch::x86::{__cpuid, __cpuid_count};
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::{__cpuid, __cpuid_count};
        let cr = unsafe { [__cpuid(1), __cpuid_count(7, 0)] };
        $($crate::__check!(cr, $tf) & )+ true
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __xgetbv {
    ($cr:expr, $mask:expr) => {{
        #[cfg(target_arch = "x86")]
        use core::arch::x86::{_xgetbv, _XCR_XFEATURE_ENABLED_MASK};
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::{_xgetbv, _XCR_XFEATURE_ENABLED_MASK};
        // check bits 26:27
        const XMASK: u32 = 0b11 << 26;
        let xsave = $cr[0].ecx & XMASK == XMASK;
        if xsave {
            let xcr0 = unsafe { _xgetbv(_XCR_XFEATURE_ENABLED_MASK) };
            (xcr0 & $mask) == $mask
        } else {
            false
        }
    }};
}

macro_rules! __generate_check {
    ($(($name:tt, $simd_reg:tt $(,$i:expr, $reg:ident, $lshift:expr)+)),+$(,)?) => {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! __check {
            $(
                ($cr:expr, $name) => {{
                    let reg_accessible = match $simd_reg {
                        "xmm" => $crate::__xgetbv!($cr, 0b10),
                        "ymm" => $crate::__xgetbv!($cr, 0b110),
                        "zmm" => $crate::__xgetbv!($cr, 0b1110_0110),
                        _ => true
                    };
                    reg_accessible $(& ($cr[$i].$reg & (1 << $lshift) != 0))+
                }};
            )+
        }
    };
}

__generate_check! {
    ("ssse3", "xmm", 0, ecx, 9),
    ("sse4.1", "xmm", 0, ecx, 19),
    ("sse2", "xmm", 0, edx, 26),
    ("avx2", "ymm", 1, ebx, 5, 0, ecx, 28),
    ("avx512ifma", "zmm", 1, ebx, 21),
    ("sha", "xmm", 1, ebx, 29),
    ("avx512vl", "zmm", 1, ebx, 31),
}
