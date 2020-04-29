

pub(crate) trait FlagHelpers {
    /// ***Z (zero)*** set to 1 when the result is equal
    /// to zero
    fn zero(&self) -> bool;
    /// ***S (sign)*** set to 1 when bit 7 (the most significant
    /// bit or MSB) of the math instruction is set
    fn sign(&self) -> bool;
    /// ***P (parity)*** is set when the answer has even parity,
    /// clear when odd parity
    fn parity(&self) -> bool;


    // /// ***CY (carry)*** set to 1 when the instruction resulted
    // /// in a carry out or borrow into the high order bit
    //  fn carry(&self) -> Option<bool>{
    //     None
    //  }

    /// ***AC (auxillary carry)*** is used mostly for
    /// BCD (binary coded decimal) math. Read the data book
    /// for more details, Space Invaders doesn't use it.
    fn aux_carry(&self) -> bool {
        //dbg!("aux_carry not implemented");
        false
    }
}

impl FlagHelpers for u8 {
    fn zero(&self) -> bool {
        *self == 0
    }

    fn sign(&self) -> bool {
        *self & 0x80 != 0
    }

    fn parity(&self) -> bool {
        self.count_ones() % 2 == 0
    }
    // fn parity(&self) -> bool {
    //     *self % 2 == 0
    // }
}

impl FlagHelpers for u16 {
    fn zero(&self) -> bool {
        *self == 0
    }

    fn sign(&self) -> bool {
        *self & 0x8000 != 0
    }

    fn parity(&self) -> bool {
       self.count_ones() % 2 == 0
    }
    // fn parity(&self) -> bool {
    //     *self % 2 == 0
    // }
}

#[cfg(test)]
mod tests {
    use super::FlagHelpers;
    #[test]
    fn cchelpers_u8() {
        assert_eq!(false, 1u8.zero());
        assert_eq!(true, 0u8.zero());

        assert_eq!(true, 0b1000_0000u8.sign());
        assert_eq!(false, 0b0100_0000u8.sign());

        assert_eq!(true, 0b1100_1100u8.parity());
        assert_eq!(false, 0b0001_1100u8.parity());
    }
    #[test]
    fn cchelpers_u16() {
        assert_eq!(false, 1u16.zero());
        assert_eq!(true, 0u16.zero());

        assert_eq!(false, (50i16 as u16).sign());
        assert_eq!(true, (-1i16 as u16).sign());

        assert_eq!(true, 0b1100_1100u16.parity());
        assert_eq!(false, 0b0001_1100u16.parity());
     }
}
