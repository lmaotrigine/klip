#[derive(Debug, Clone, Copy)]
pub struct Params {
    pub(crate) log_n: u8,
    pub(crate) r: u32,
    pub(crate) p: u32,
}

impl Params {
    #[allow(clippy::cast_possible_truncation, clippy::checked_conversions)]
    pub fn new(log_n: u8, r: u32, p: u32) -> Result<Self, &'static str> {
        let cond1 = (log_n as usize) < usize::BITS as usize;
        let cond2 = core::mem::size_of::<usize>() >= core::mem::size_of::<u32>();
        #[allow(clippy::cast_possible_truncation)]
        let cond3 = r <= usize::MAX as u32 && p < usize::MAX as u32;
        if !(r > 0 && p > 0 && cond1 && (cond2 || cond3)) {
            return Err("invalid parameters");
        }
        let r = r as usize;
        let p = p as usize;
        let n = 1 << log_n;
        let r128 = r.checked_mul(128).ok_or("invalid parameters")?;
        r128.checked_mul(p).ok_or("invalid parameters")?;
        r128.checked_mul(n).ok_or("invalid parameters")?;
        if (log_n as usize) >= r * 16 {
            return Err("invalid parameters");
        }
        if r * p >= 0x4000_0000 {
            return Err("invalid parameters");
        }
        Ok(Self {
            log_n,
            r: r as u32,
            p: p as u32,
        })
    }
}
