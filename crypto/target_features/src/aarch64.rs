//! ARM64 CPU feature detection support.
//!
//! Unfortunately the ARM instruction to detect CPU features cannot be called
//! from unprivileged userspace code, so this implementation relies on
//! OS-specific APIs.

#[macro_export]
#[doc(hidden)]
macro_rules! __unless {
    ($($tf:tt),+ => $body:expr) => {{
        #[cfg(not(all($(target_feature = $tf,)+)))]
        $body
        #[cfg(all($(target_feature = $tf,)+))]
        true
    }};
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[macro_export]
#[doc(hidden)]
macro_rules! __detect {
    ($($tf:tt),+) => {{
        let hwcaps = $crate::aarch64::hwcaps();
        $($crate::__check!(hwcaps, $tf) & )+ true
    }};
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[doc(hidden)]
#[allow(dead_code)]
#[must_use]
pub fn hwcaps() -> u64 {
    unsafe { libc::getauxval(libc::AT_HWCAP) }
}

#[cfg(target_vendor = "apple")]
#[macro_export]
#[doc(hidden)]
macro_rules! __detect {
    ($($tf:tt),+) => {{
        $($crate::__check!($tf) & )+ true
    }};
}

#[cfg(any(target_os = "android", target_os = "linux"))]
macro_rules! __generate_check {
    ($(($name:tt, $cap:ident)),+$(,)?) => {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! __check {
            $(
                ($caps:expr, $name) => {
                    (($caps & $crate::aarch64::caps::$cap) != 0)
                };
            )+
        }
    };
}

#[cfg(any(target_os = "android", target_os = "linux"))]
__generate_check! {
    ("sha2", SHA2),
    ("sha3", SHA3),
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[doc(hidden)]
#[allow(dead_code)]
pub mod caps {
    pub const SHA2: u64 = libc::HWCAP_SHA2;
    pub const SHA3: u64 = libc::HWCAP_SHA3 | libc::HWCAP_SHA512;
}

#[cfg(target_vendor = "apple")]
#[macro_export]
#[doc(hidden)]
macro_rules! __check {
    ("sha2") => {
        true
    };
    ("sha3") => {
        unsafe {
            $crate::aarch64::sysctlbyname(b"hw.optional.armv8_2_sha512\0")
                && $crate::aarch64::sysctlbyname(b"hw.optional.armv8_2_sha3\0")
        }
    };
}

#[cfg(target_vendor = "apple")]
#[doc(hidden)]
pub unsafe fn sysctlbyname(name: &[u8]) -> bool {
    let mut val = 0;
    let val_ptr: *mut u32 = &mut val;
    let mut size = core::mem::size_of::<u32>();
    let ret = libc::sysctlbyname(
        name.as_ptr().cast(),
        val_ptr.cast(),
        &mut size,
        core::ptr::null_mut(),
        0,
    );
    assert_eq!(size, 4, "unexpected sysctlbyname(3) result size");
    assert_eq!(ret, 0, "sysctlbyname(3) returned error code: {ret}");
    val != 0
}

#[cfg(not(any(target_os = "android", target_os = "linux", target_vendor = "apple")))]
#[macro_export]
#[doc(hidden)]
macro_rules! __detect {
    ($($tf:tt),+) => {
        false
    };
}
