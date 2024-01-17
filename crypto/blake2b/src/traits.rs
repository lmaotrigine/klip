pub trait SliceExt {
    fn set_bytes(&mut self, value: u8);
    fn copy_bytes_from(&mut self, src: &[u8]);
}

impl SliceExt for [u8] {
    #[inline]
    fn set_bytes(&mut self, value: u8) {
        unsafe { core::ptr::write_bytes(self.as_mut_ptr(), value, self.len()) }
    }

    #[inline]
    fn copy_bytes_from(&mut self, src: &[u8]) {
        assert!(self.len() >= src.len());
        unsafe { core::ptr::copy_nonoverlapping(src.as_ptr(), self.as_mut_ptr(), src.len()) }
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for i8 {}
    impl Sealed for i16 {}
    impl Sealed for i32 {}
    impl Sealed for i64 {}
    impl Sealed for crate::simd::u64x4 {}
    impl<T: Sealed> Sealed for [T] {}
}

pub trait AsBytes: sealed::Sealed {
    fn as_bytes(&self) -> &[u8];
    fn as_mut_bytes(&mut self) -> &mut [u8];
}

impl<T: sealed::Sealed> AsBytes for [T] {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.as_ptr().cast(), core::mem::size_of_val(self)) }
    }

    #[inline]
    fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self.as_mut_ptr().cast(), core::mem::size_of_val(self))
        }
    }
}
