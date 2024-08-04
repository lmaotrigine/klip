macro_rules! as_array {
    ($arr:expr, $offset:expr, $len:expr) => {{
        {
            #[inline]
            #[allow(clippy::ptr_as_ptr)]
            const unsafe fn as_array<T>(arr: &[T]) -> &[T; $len] {
                &*(arr.as_ptr() as *const [T; $len])
            }
            let offset = $offset;
            let arr = &$arr[offset..offset + $len];
            unsafe { as_array(arr) }
        }
    }};
}
pub(crate) use as_array;

macro_rules! as_arrays {
    ($arr:expr, $($len:expr),+) => {{
        {
            #[inline]
            #[allow(clippy::ptr_as_ptr, unused_assignments)]
            const unsafe fn as_arrays<T>(a: &[T; $($len + )+ 0]) -> ($(&[T;
$len],)+) {                 let mut p = a.as_ptr();
                (
                    $({
                        let a_ = &*(p as *const [T; $len]);
                        p = p.add($len);
                        a_
                    },)+
                )
            }
            let inp = $arr;
            unsafe { as_arrays(inp) }
        }
    }};
}
pub(crate) use as_arrays;

macro_rules! as_arrays_mut {
    ($arr:expr, $($len:expr),+) => {{
        {
            #[inline]
            #[allow(clippy::ptr_as_ptr, unused_assignments)]
            unsafe fn as_arrays_mut<T>(a: &mut [T; $($len + )+ 0]) -> ($(&mut
[T; $len],)*) {                 let mut p = a.as_mut_ptr();
                (
                    $({
                        let a_ = &mut *(p as *mut [T; $len]);
                        p = p.add($len);
                        a_
                    },)+
                )
            }
            let inp = $arr;
            unsafe { as_arrays_mut(inp) }
        }
    }};
}

pub(crate) use as_arrays_mut;
