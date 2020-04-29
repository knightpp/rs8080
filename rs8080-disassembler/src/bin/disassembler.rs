use rs8080_disassembler::*;
use std::fs::File;
use std::io::Read;

fn main() {
    let buf = include_bytes!("../../../roms/cpudiag.bin");
    let mut byte: usize = 0;
    for cmd in Command::iterator(buf) {
        print!("{:04X} ", byte);
        println!("{}", cmd);
        byte += cmd.size as usize;
    }
}
