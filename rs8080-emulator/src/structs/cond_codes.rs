use crate::traits::FlagHelpers;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Default)]
/// Represents 8 bit flag register
pub(crate) struct ConditionalCodes {
    /// Zero flag, set when result is zero
    pub(crate) z: bool,
    /// Sign flag, set when result is negative
    pub(crate) s: bool,
    /// Parity flag, set when number of bits
    /// in the result is odd
    pub(crate) p: bool,
    /// Carry flag, set when high/low bit shifts to low/high
    pub(crate) cy: bool,
    /// Aux carry, not implemented
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
    /// Sets conditional codes
    /// # Registers affected
    /// Z, S, P. AC - not implemented
    pub fn set_zspac<T>(&mut self, val: T)
    where
        T: FlagHelpers,
    {
        self.z = val.zero();
        self.s = val.sign();
        self.p = val.parity();
        self.ac = val.aux_carry();
    }
    /// Sets conditional codes according to CMP operation
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
