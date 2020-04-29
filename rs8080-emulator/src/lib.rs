//extern crate rs8080_disassembler as disasm;

mod structs;
mod traits;

pub use crate::traits::DataBus;
pub use structs::RS8080;
pub use traits::{MemLimiter, WriteAction};
