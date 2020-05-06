use crate::structs::rs8080::AllowAll;

/// Action that happens on mem write
pub enum WriteAction {
    /// Alow mem write
    Allow,
    /// Intercept and write the byte
    NewByte(u8),
    /// Do nothing
    Ignore,
}
/// Can be used to intercept mem access or block reads/writes
/// to specific mem locations
pub trait MemLimiter {
    fn check_write(&self, adr: u16, to_write_byte: u8) -> WriteAction;
    fn check_read(&self, adr: u16, read_byte: u8) -> u8;
}
