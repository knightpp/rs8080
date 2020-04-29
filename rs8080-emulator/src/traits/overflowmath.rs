use crate::traits::HiPart;

pub(crate) trait OverflowMath {
    type RHS;
    fn add_carry(&mut self, rhs: Self::RHS) -> bool;
    fn sub_carry(&mut self, rhs: Self::RHS) -> bool;
    fn add_un(&mut self, rhs: Self::RHS);
    fn sub_un(&mut self, rhs: Self::RHS);
}

impl OverflowMath for u8 {
    type RHS = Self;
    fn add_carry(&mut self, rhs: Self::RHS) -> bool {
        let x = *self as u16 + rhs as u16;
        *self = x as u8;
        x.get_hipart() > 0
    }

    fn sub_carry(&mut self, rhs: Self::RHS) -> bool {
        let carry = *self < rhs;
        *self = self.wrapping_sub(rhs);
        carry
    }

    fn add_un(&mut self, rhs: Self::RHS) {
        *self = self.wrapping_add(rhs);
    }
    fn sub_un(&mut self, rhs: Self::RHS) {
        *self = self.wrapping_sub(rhs);
    }
}

impl OverflowMath for u16 {
    type RHS = Self;
    fn add_carry(&mut self, rhs: Self::RHS) -> bool {
        let x = *self as u32 + rhs as u32;
        *self = x as u16;
        x.get_hipart() > 0
    }

    fn sub_carry(&mut self, rhs: Self::RHS) -> bool {
        let carry = *self < rhs;
        *self = self.wrapping_sub(rhs);
        carry
    }

    fn add_un(&mut self, rhs: Self::RHS) {
        *self = self.wrapping_add(rhs);
    }
    fn sub_un(&mut self, rhs: Self::RHS) {
        *self = self.wrapping_sub(rhs);
    }
}
