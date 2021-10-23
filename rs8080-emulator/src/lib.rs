//extern crate rs8080_disassembler as disasm;

mod structs;
mod traits;

pub use crate::traits::DataBus;
pub use structs::RS8080;
pub use traits::{MemLimiter, WriteAction};

extern crate derive_more;
use derive_more::{Add, Display, From};

#[derive(Add, Display, From)]
pub struct ClockCycles(pub u32);

impl ClockCycles {
    pub(crate) fn add(&mut self, cycles: u32) {
        self.0 += cycles;
    }
}
