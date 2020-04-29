#[derive(Copy,Clone)]
pub(crate) struct TwoU8 {
    pub(crate) hi: u8,
    pub(crate) lo: u8,
}

impl TwoU8 {
    pub(crate) fn new(lo: u8, hi: u8) -> TwoU8 {
        TwoU8 { lo, hi }
    }
}

impl From<TwoU8> for usize{
    fn from(x: TwoU8) -> Self {
        let x : u16 = x.into();
        x as usize
    }
}

impl From<u16> for TwoU8 {
    fn from(x: u16) -> Self {
        TwoU8 {
            lo: x as u8,
            hi: (x >> 8) as u8,
        }
    }
}

impl From<TwoU8> for u16{
    fn from(x: TwoU8) -> Self {
        ((x.hi as u16) << 8) | x.lo as u16
    }
}
