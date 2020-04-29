
pub enum WriteAction{
    Allow,
    NewByte(u8),
    Ignore,
}
pub trait MemLimiter{
    fn check_write(&self, adr : u16, to_write_byte : u8) -> WriteAction;
    fn check_read(&self, adr : u16, read_byte : u8) -> u8;
}

