use std::fmt::{self, Display, Formatter};
use std::{num::Wrapping, ops};

use crate::traits::{FlagHelpers, OverflowMath};
use ops::Sub;

#[derive(Debug)]
pub(crate) struct ConditionalCodes {
    pub(crate) z: bool,
    pub(crate) s: bool,
    pub(crate) p: bool,
    pub(crate) cy: bool,
    pub(crate) ac: bool,
}

impl Display for ConditionalCodes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        macro_rules! flagify {
            ($name:ident) => {
                if self.$name {
                    stringify!($name)
                } else {
                    "."
                }
            };
        }
        write!(
            f,
            "{}{}{}{}",
            flagify!(z),
            flagify!(s),
            flagify!(p),
            if self.cy { "c" } else { "." },
            // flagify!(self, ac)
        )
    }
}

impl ConditionalCodes {
    /// Value size must be x2 greater
    /// Example: eu8 -> u16
    /// # Registers affected
    /// Z, S, P, AC
    pub fn set_zspac<T>(&mut self, val: T)
    where
        T: FlagHelpers,
    {
        self.z = val.zero();
        self.s = val.sign();
        self.p = val.parity();
        self.ac = val.aux_carry();
    }

    /// #Registers affected
    /// Z, S, P, CY
    pub fn set_cmp(&mut self, lhs: u8, rhs: u8) {
        let x = lhs.wrapping_sub(rhs);
        self.z = lhs == rhs;
        self.cy = lhs < rhs;
        self.s = x.sign();
        self.p = x.parity();
    }
}
