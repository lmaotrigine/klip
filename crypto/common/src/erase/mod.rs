#![allow(clippy::module_name_repetitions)]

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

#[inline(always)]
fn atomic_fence() {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

#[inline(always)]
fn volatile_write<T: Copy + Sized>(src: T, dst: &mut T) {
    unsafe { core::ptr::write_volatile(dst, src) }
}

pub trait Erase {
    fn erase(&mut self);
}

pub trait EraseOnDrop {}

trait DefaultIsErased: Copy + Default + Sized {}

impl<E: DefaultIsErased> Erase for E {
    fn erase(&mut self) {
        volatile_write(E::default(), self);
        atomic_fence();
    }
}

macro_rules! impl_default_is_erased {
    ($($t:ty),*) => {
        $(
            impl DefaultIsErased for $t {}
        )*
    };
}

#[rustfmt::skip]
impl_default_is_erased! {
    core::marker::PhantomPinned,(), bool, char,
    f32, f64,
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize
}

impl EraseOnDrop for core::marker::PhantomPinned {}
impl EraseOnDrop for () {}

impl<E: Erase, const N: usize> Erase for [E; N] {
    fn erase(&mut self) {
        self.iter_mut().erase();
    }
}

impl<E: EraseOnDrop, const N: usize> EraseOnDrop for [E; N] {}

impl<E: Erase> Erase for core::slice::IterMut<'_, E> {
    fn erase(&mut self) {
        for elem in self {
            elem.erase();
        }
    }
}
