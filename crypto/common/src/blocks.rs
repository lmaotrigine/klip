pub type Block<const N: usize> = [u8; N];

#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct Buffer<const N: usize> {
    buffer: Block<N>,
    pos: u8,
}

impl<const N: usize> Default for Buffer<N> {
    fn default() -> Self {
        Self {
            buffer: [0; N],
            pos: 0,
        }
    }
}

impl<const N: usize> Clone for Buffer<N> {
    #[allow(clippy::clone_on_copy)]
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            pos: self.pos,
        }
    }
}

impl<const N: usize> Buffer<N> {
    #[inline]
    pub fn digest_blocks(&mut self, mut input: &[u8], mut compress: impl FnMut(&[Block<N>])) {
        let pos = self.get_pos();
        let rem = N - pos;
        let n = input.len();
        if n < rem {
            self.buffer[pos..][..n].copy_from_slice(input);
            self.set_pos(pos + n);
            return;
        }
        if pos != 0 {
            let (left, right) = input.split_at(rem);
            input = right;
            self.buffer[pos..].copy_from_slice(left);
            compress(core::slice::from_ref(&self.buffer));
        }
        let (blocks, left) = Self::split_blocks(input);
        if !blocks.is_empty() {
            compress(blocks);
        }
        let n = left.len();
        self.buffer[..n].copy_from_slice(left);
        self.set_pos(n);
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        self.set_pos(0);
    }

    #[inline(always)]
    #[must_use]
    pub fn get_pos(&self) -> usize {
        let pos = self.pos as usize;
        if pos >= N {
            debug_assert!(false);
            unsafe {
                core::hint::unreachable_unchecked();
            }
        }
        pos
    }

    #[inline(always)]
    #[allow(clippy::cast_possible_truncation)]
    fn set_pos(&mut self, pos: usize) {
        debug_assert!(pos <= N);
        self.pos = pos as u8;
    }

    #[inline]
    pub fn len64_padding_be(&mut self, data_len: u64, mut compress: impl FnMut(&Block<N>)) {
        let pos = self.get_pos();
        self.buffer[pos] = 0x80;
        let suffix = &data_len.to_be_bytes();
        for b in &mut self.buffer[pos + 1..] {
            *b = 0;
        }
        if N - pos - 1 < 8 {
            compress(&self.buffer);
            let mut block = [0; N];
            block[N - 8..].copy_from_slice(suffix);
            compress(&block);
        } else {
            self.buffer[N - 8..].copy_from_slice(suffix);
            compress(&self.buffer);
        }
        self.set_pos(0);
    }

    #[inline]
    pub fn len128_padding_be(&mut self, data_len: u128, mut compress: impl FnMut(&Block<N>)) {
        let pos = self.get_pos();
        self.buffer[pos] = 0x80;
        let suffix = &data_len.to_be_bytes();
        for b in &mut self.buffer[pos + 1..] {
            *b = 0;
        }
        if N - pos - 1 < 16 {
            compress(&self.buffer);
            let mut block = [0; N];
            block[N - 16..].copy_from_slice(suffix);
            compress(&block);
        } else {
            self.buffer[N - 16..].copy_from_slice(suffix);
            compress(&self.buffer);
        }
        self.set_pos(0);
    }

    #[inline(always)]
    const fn split_blocks(data: &[u8]) -> (&[Block<N>], &[u8]) {
        let nb = data.len() / N;
        let blocks_len = nb * N;
        let tail_len = data.len() - blocks_len;
        unsafe {
            let blocks_ptr = data.as_ptr().cast::<Block<N>>();
            let tail_ptr = data.as_ptr().add(blocks_len);
            (
                core::slice::from_raw_parts(blocks_ptr, nb),
                core::slice::from_raw_parts(tail_ptr, tail_len),
            )
        }
    }
}

impl<const N: usize> super::erase::Erase for Buffer<N> {
    fn erase(&mut self) {
        self.buffer.erase();
        self.pos.erase();
    }
}
