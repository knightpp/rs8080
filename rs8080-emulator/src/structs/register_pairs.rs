use crate::structs::TwoU8;
use crate::traits::{HiPart, OverflowMath};
use std::ops::{AddAssign, SubAssign};

fn merge(lo: u8, hi: u8) -> u16 {
    TwoU8 { lo, hi }.into()
}

macro_rules! impl_ops {
    ($struct_name:ident, $hi:ident, $lo:ident) => {
        impl AddAssign<u16> for $struct_name {
            fn add_assign(&mut self, rhs: u16) {
                let val = merge(self.$lo, self.$hi).wrapping_add(rhs);
                self.$hi = (val >> 8) as u8;
                self.$lo = (val) as u8;
            }
        }

        impl SubAssign<u16> for $struct_name {
            fn sub_assign(&mut self, rhs: u16) {
                let val = merge(self.$lo, self.$hi).wrapping_sub(rhs);
                self.$hi = (val >> 8) as u8;
                self.$lo = (val) as u8;
            }
        }

        impl $struct_name {
            pub(crate) fn get_twou8(&self) -> TwoU8 {
                TwoU8::new(self.$lo, self.$hi)
            }

            pub(crate) fn set(&mut self, value: impl Into<TwoU8>) {
                let x: TwoU8 = value.into();
                self.$hi = x.hi;
                self.$lo = x.lo;
            }
        }

        impl From<$struct_name> for TwoU8 {
            fn from(x: $struct_name) -> Self {
                TwoU8 {
                    lo: x.$lo,
                    hi: x.$hi,
                }
            }
        }

        impl From<$struct_name> for usize {
            fn from(x: $struct_name) -> Self {
                TwoU8 {
                    lo: x.$lo,
                    hi: x.$hi,
                }
                .into()
            }
        }

        impl From<$struct_name> for u16 {
            fn from(x: $struct_name) -> u16 {
                TwoU8 {
                    lo: x.$lo,
                    hi: x.$hi,
                }
                .into()
            }
        }
        impl From<u16> for $struct_name {
            fn from(x: u16) -> Self {
                $struct_name {
                    $hi: (x >> 8) as u8,
                    $lo: x as u8,
                }
            }
        }

        impl OverflowMath for $struct_name {
            type RHS = u16;

            fn add_carry(&mut self, rhs: Self::RHS) -> bool {
                let mut x = u16::from(*self);
                let carry = x.add_carry(rhs);
                self.set(x);
                carry
            }

            fn sub_carry(&mut self, rhs: Self::RHS) -> bool {
                let mut x = u16::from(*self);
                let carry = x.sub_carry(rhs);
                self.set(x);
                carry
            }

            fn add_un(&mut self, rhs: Self::RHS) {
                let mut x = u16::from(*self);
                x.add_un(rhs);
                self.set(x);
            }

            fn sub_un(&mut self, rhs: Self::RHS) {
                let mut x = u16::from(*self);
                x.sub_un(rhs);
                self.set(x);
            }
        }
    };
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct BC {
    pub(crate) b: u8,
    pub(crate) c: u8,
}
#[derive(Copy, Clone, Debug)]
pub(crate) struct DE {
    pub(crate) d: u8,
    pub(crate) e: u8,
}
#[derive(Copy, Clone, Debug)]
pub(crate) struct HL {
    pub(crate) h: u8,
    pub(crate) l: u8,
}

impl_ops!(BC, b, c);
impl_ops!(DE, d, e);
impl_ops!(HL, h, l);
