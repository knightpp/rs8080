/// Does work when `IN` and `OUT` commands is executing
pub trait DataBus {
    /// __Reads__ value __from__ port, invoked by `IN`
    fn port_in(&mut self, port: u8) -> u8;
    /// __Writes__ value __to__ port, invoked by `OUT`
    fn port_out(&mut self, rega: u8, port: u8);

    fn port(&mut self, index: usize) -> &mut u8;
}
