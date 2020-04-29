use rs8080_emulator::{RS8080, DataBus};

fn print_mem(mem : &[u8], pc: u16,bytes_after : u16){
    for (i, line) in mem[pc as usize..(pc + bytes_after) as usize]
        .chunks(14).enumerate()
    {
        println!("{:04X} | {:02X?}", i * 0xF + pc as usize, line);
    }
}
struct DummyIO{}
impl DataBus for DummyIO{
    fn port_in(&mut self, _ : u8) -> u8 { todo!() }
    fn port_out(&mut self, _ : u8, _ : u8) { todo!() }
    fn port(&mut self, _ : usize) -> &mut u8 { todo!() }
}

// fn main() {
//     let mut emu = RS8080::new(Box::new(DummyIO{}));
//     //let file = include_bytes!(r"..\..\roms\invaders.rom");
//     let h = include_bytes!("../../roms/invaders.h");
//     let g = include_bytes!("../../roms/invaders.g");
//     let f = include_bytes!("../../roms/invaders.f");
//     let e = include_bytes!("../../roms/invaders.e");
//     emu.load_to_mem(h, 0);
//     emu.load_to_mem(g, 0x0800);
//     emu.load_to_mem(f, 0x1000);
//     emu.load_to_mem(e, 0x1800);

//     let mut i = 0;
//     const n : usize = 1600;
//     for z in 0..=n{
//         if z > n - 100{
//             let s = format!("{}", emu.disassemble_next());
//             println!("{:04} | {:028} |{:>}", i, s, emu);
//         }
//         emu.emulate_next();
//         i += 1;
//     }

//         //let mut i = 0;
//     let mut str = String::new();
//     loop{
//         std::io::stdin().read_line(&mut str).unwrap();
//         let s = format!("{}", emu.disassemble_next());
//         println!("{:04} | {:028} |{:>}", i, s, emu);
//         emu.emulate_next();
//         i += 1;
//     }
// }


fn main() {
    let mut emu = RS8080::new(Box::new(DummyIO{}));
    //let file = include_bytes!(r"..\..\roms\invaders.rom");
    let bin = include_bytes!("../../roms/cpudiag.bin");

    emu.load_to_mem(bin, 0x100);

    emu.get_mut_mem()[0x59c] = 0xc3; //JMP    
    emu.get_mut_mem()[0x59d] = 0xc2;    
    emu.get_mut_mem()[0x59e] = 0x05;
    emu.get_mut_mem()[368] = 0x7;  
    let mut i = 0;
    // const n : usize = 150;
    // for z in 0..=n{
    //     if z > n - 100{
    //         let s = format!("{}", emu.disassemble_next());
    //         println!("{:04} | {:028} |{:>}", i, s, emu);
    //     }
    //     emu.emulate_next();
    //     i += 1;
    // }

        //let mut i = 0;
    let mut str = String::new();
    loop{
        //std::io::stdin().read_line(&mut str).unwrap();
        let s = format!("{}", emu.disassemble_next());
        println!("{:04} | {:028} |{:>}", i, s, emu);
        emu.emulate_next();
        i += 1;
    }
}

