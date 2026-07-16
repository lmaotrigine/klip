use core::sync::atomic;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SafeString {
    inner: String,
}

impl SafeString {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }

    #[must_use]
    pub fn into_inner(mut self) -> String {
        core::mem::take(&mut self.inner)
    }
}

impl Drop for SafeString {
    fn drop(&mut self) {
        let default = u8::default();

        for c in unsafe { self.inner.as_bytes_mut() } {
            unsafe { core::ptr::write_volatile(c, default) };
        }

        atomic::fence(atomic::Ordering::SeqCst);
        atomic::compiler_fence(atomic::Ordering::SeqCst);
    }
}

impl core::ops::Deref for SafeString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::DerefMut for SafeString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<String> for SafeString {
    fn from(value: String) -> Self {
        Self { inner: value }
    }
}
