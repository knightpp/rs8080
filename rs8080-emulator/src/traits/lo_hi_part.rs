pub(crate) trait HiPart {
    type Output;
    fn get_hipart(&self) -> Self::Output;
}

impl HiPart for u32 {
    type Output = u16;
    fn get_hipart(&self) -> Self::Output {
        //(*self & !0xFFFF_u32 >> 16) as u16
        (*self >> 16) as u16
    }
}

impl HiPart for u16 {
    type Output = u8;
    fn get_hipart(&self) -> Self::Output {
        //(*self & !0xFF_u16 >> 8) as u8
        (*self >> 8) as u8
    }
}

pub(crate) trait LoPart {
    type Output;
    fn get_lopart(&self) -> Self::Output;
}

impl LoPart for u32 {
    type Output = u16;
    fn get_lopart(&self) -> Self::Output {
        *self as u16
    }
}

impl LoPart for u16 {
    type Output = u8;
    fn get_lopart(&self) -> Self::Output {
        *self as u8
    }
}
