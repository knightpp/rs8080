extern crate rs8080_emulator as emulator;
use emulator::{MemLimiter, WriteAction};

pub(crate) struct SpaceInvadersLimit {}
impl MemLimiter for SpaceInvadersLimit {
    fn check_write(&self, adr: u16, _: u8) -> WriteAction {
        if adr < 0x2000 {
            eprintln!("block: write mem < 0x2000");
            WriteAction::Ignore
        } else if adr >= 0x4000 {
            eprintln!("block: write mem >= 0x4000");
            WriteAction::Ignore
        } else {
            WriteAction::Allow
        }
    }
    fn check_read(&self, _: u16, read_byte: u8) -> u8 {
        read_byte
    }
}
