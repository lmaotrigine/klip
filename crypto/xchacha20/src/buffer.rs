use core::marker::PhantomData;

#[allow(clippy::module_name_repetitions)]
pub struct MutableBuffer<'a, T> {
    in_ptr: *const T,
    out_ptr: *mut T,
    len: usize,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T> MutableBuffer<'a, T> {
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn get(&mut self, pos: usize) -> Mut<'_, T> {
        debug_assert!(pos < self.len);
        Mut {
            in_ptr: unsafe { self.in_ptr.add(pos) },
            out_ptr: unsafe { self.out_ptr.add(pos) },
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn split_at(self, mid: usize) -> (Self, Self) {
        debug_assert!(mid <= self.len);
        let (tail_in_ptr, tail_out_ptr) = unsafe { (self.in_ptr.add(mid), self.out_ptr.add(mid)) };
        (
            MutableBuffer {
                in_ptr: self.in_ptr,
                out_ptr: self.out_ptr,
                len: mid,
                _marker: PhantomData,
            },
            MutableBuffer {
                in_ptr: tail_in_ptr,
                out_ptr: tail_out_ptr,
                len: self.len() - mid,
                _marker: PhantomData,
            },
        )
    }

    #[inline(always)]
    pub const fn into_chunks<const N: usize>(self) -> (MutableBuffer<'a, [T; N]>, Self) {
        let chunks = self.len() / N;
        let tail_pos = N * chunks;
        let tail_len = self.len() - tail_pos;
        let chunks = MutableBuffer {
            in_ptr: self.in_ptr.cast::<[T; N]>(),
            out_ptr: self.out_ptr.cast::<[T; N]>(),
            len: chunks,
            _marker: PhantomData,
        };
        let tail = MutableBuffer {
            in_ptr: unsafe { self.in_ptr.add(tail_pos) },
            out_ptr: unsafe { self.out_ptr.add(tail_pos) },
            len: tail_len,
            _marker: PhantomData,
        };
        (chunks, tail)
    }
}

impl<'a> MutableBuffer<'a, u8> {
    #[inline(always)]
    pub fn xor(&mut self, data: &[u8]) {
        assert_eq!(self.len(), data.len());
        unsafe {
            for (i, datum) in data.iter().enumerate() {
                let in_ptr = self.in_ptr.add(i);
                let out_ptr = self.out_ptr.add(i);
                *out_ptr = *in_ptr ^ *datum;
            }
        }
    }
}

impl<'a, T> From<&'a mut [T]> for MutableBuffer<'a, T> {
    #[inline(always)]
    fn from(value: &'a mut [T]) -> Self {
        let p = value.as_mut_ptr();
        Self {
            in_ptr: p,
            out_ptr: p,
            len: value.len(),
            _marker: PhantomData,
        }
    }
}

impl<'a, T> IntoIterator for MutableBuffer<'a, T> {
    type IntoIter = MutableBufferIter<'a, T>;
    type Item = Mut<'a, T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        MutableBufferIter { buf: self, pos: 0 }
    }
}

pub struct Mut<'a, T> {
    in_ptr: *const T,
    out_ptr: *mut T,
    _marker: PhantomData<&'a ()>,
}

impl<'a, const N: usize> Mut<'a, [u8; N]> {
    #[inline(always)]
    pub fn xor(&mut self, data: &[u8; N]) {
        unsafe {
            let inp = core::ptr::read(self.in_ptr);
            let mut tmp = [0; N];
            for i in 0..N {
                tmp[i] = inp[i] ^ data[i];
            }
            core::ptr::write(self.out_ptr, tmp);
        }
    }
}

impl<'a, const N: usize, const M: usize> Mut<'a, [[u8; N]; M]> {
    #[inline(always)]
    pub fn xor(&mut self, data: &[[u8; N]; M]) {
        unsafe {
            let inp = core::ptr::read(self.in_ptr);
            let mut tmp = [[0; N]; M];
            for i in 0..M {
                for j in 0..N {
                    tmp[i][j] = inp[i][j] ^ data[i][j];
                }
            }
            core::ptr::write(self.out_ptr, tmp);
        }
    }
}

pub struct MutableBufferIter<'a, T> {
    buf: MutableBuffer<'a, T>,
    pos: usize,
}

impl<'a, T> Iterator for MutableBufferIter<'a, T> {
    type Item = Mut<'a, T>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() == self.pos {
            return None;
        }
        let res = unsafe {
            Mut {
                in_ptr: self.buf.in_ptr.add(self.pos),
                out_ptr: self.buf.out_ptr.add(self.pos),
                _marker: PhantomData,
            }
        };
        self.pos += 1;
        Some(res)
    }
}
