use std::fmt::{self, Display, Formatter};
use Argument::*;

#[derive(IntoStaticStr)]
pub enum Argument {
    /// lo, hi
    Addr(u8, u8),
    D8(u8),
    /// lo, hi
    D16(u8, u8),

    // Registers
    PSW,
    M,
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    SP,
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Addr(lo, hi) => write!(f, "${:02X}{:02X}", hi, lo),
            D8(d8) => write!(f, "#__{:<02X}", d8),
            D16(lo, hi) => write!(f, "#{:02X}{:02X}", hi, lo),
            _ => {
                let s: &'static str = self.into();
                write!(f, "{}", s)
            }
        }
    }
}

impl From<Argument> for Vec<Argument> {
    fn from(x: Argument) -> Self {
        vec![x]
    }
}

pub(crate) trait MyToString {
    fn to_string(&self) -> String;
}

impl MyToString for Vec<Argument> {
    fn to_string(&self) -> String {
        self.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}
