use rs8080_emulator::{DataBus, RS8080};

#[allow(dead_code)]
fn print_mem(mem: &[u8], pc: u16, bytes_after: u16) {
    for (i, line) in mem[pc as usize..(pc + bytes_after) as usize]
        .chunks(14)
        .enumerate()
    {
        println!("{:04X} | {:02X?}", i * 0xF + pc as usize, line);
    }
}
struct DummyIO {}
impl DataBus for DummyIO {
    fn port_in(&mut self, _: u8) -> u8 {
        todo!()
    }
    fn port_out(&mut self, _: u8, _: u8) {
        todo!()
    }
    fn port(&mut self, _: usize) -> &mut u8 {
        todo!()
    }
}

fn main() {
    let mut emu = RS8080::new(DummyIO {});
    let bin = include_bytes!("../../roms/cpudiag.bin");

    emu.load_to_mem(bin, 0x100);

    emu.get_mut_mem()[0x59c] = 0xc3; //JMP
    emu.get_mut_mem()[0x59d] = 0xc2;
    emu.get_mut_mem()[0x59e] = 0x05;
    //emu.get_mut_mem()[368] = 0x7;
    let mut i = 0;
    loop {
        let s = format!("{}", emu.disassemble_next());
        println!("{:04} | {:028} |{:>}", i, s, emu);
        emu.emulate_next();
        i += 1;
    }
}
