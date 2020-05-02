extern crate rs8080_emulator as emulator;
extern crate sdl2;
use emulator::{DataBus, RS8080};
use emulator::{MemLimiter, WriteAction};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, WindowCanvas};
use std::{
    thread,
    time::{Duration, Instant},
};

fn get_bitvec(byte: u8) -> [bool; 8] {
    let mut bitvec = [false; 8];
    bitvec[0] = byte & 0b0000_0001 > 0;
    bitvec[1] = byte & 0b0000_0010 > 0;
    bitvec[2] = byte & 0b0000_0100 > 0;
    bitvec[3] = byte & 0b0000_1000 > 0;
    bitvec[4] = byte & 0b0001_0000 > 0;
    bitvec[5] = byte & 0b0010_0000 > 0;
    bitvec[6] = byte & 0b0100_0000 > 0;
    bitvec[7] = byte & 0b1000_0000 > 0;
    bitvec
}

struct SpaceInvadersLimit {}
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

pub struct SpaceInvadersIO {
    ports: [u8; 6],
    shift0: u8,
    shift1: u8,
    shift_offset: u8,
}

impl SpaceInvadersIO {
    pub fn new() -> SpaceInvadersIO {
        SpaceInvadersIO {
            ports: [0; 6],
            shift0: 0,
            shift1: 0,
            shift_offset: 0,
        }
    }

    pub fn set_shift_offset(&mut self, offset : u8){
        self.ports[2] = offset & 0x7;
    }
}

impl DataBus for SpaceInvadersIO {
    fn port_in(&mut self, port: u8) -> u8 {
        match port {
            0 => 0xf,
            1 => self.ports[1],
            2 => 0,
            3 => {
                //let v: u16 = ((self.shift1 as u16) << 8) | self.shift0 as u16;
                //((v >> (8u8 - self.shift_offset) as u16) & 0xFF) as u8
                (((self.shift0 as u16) << 8) | self.shift1 as u16)
                    .rotate_left(self.shift_offset as u32) as u8
            }
            _ => 0,
        }
    }

    fn port_out(&mut self, value: u8, port: u8) {
        match port {
            2 => {
                self.shift_offset = value & 0b0000_0111u8;
            }
            3 => self.ports[3] = value,
            4 => {
                self.shift0 = self.shift1;
                self.shift1 = value;
            }
            5 => self.ports[5] = value,
            _ => {}
        }
    }

    fn port(&mut self, index: usize) -> &mut u8 {
        &mut self.ports[index]
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn invaders_shift_register(){
        let mut io = SpaceInvadersIO::new();
        io.port_out(0xFF, 4); // write 0xFF to shift1
        assert_eq!(0xFF, io.shift1);
        io.port_out(0, 2);  // set offset to 0
        assert_eq!(io.shift_offset, 0);
        io.port_out(0b0000_0111, 2); // set shift_offset to 7
        assert_eq!(io.shift_offset, 7);
        assert_eq!(io.port_in(3), 0xFF << 7);

        io.port_out(13, 4); // write 13 to shift1, shift0 = 0xFF
        io.port_out(3, 2); // set shift_offset to 3
        assert_eq!(io.port_in(3), (0x0DFF >> (8 - 3)) as u8);
    }

}


pub fn draw_space_invaders_vram(canvas: &mut WindowCanvas, tex: &mut Texture, vram: &[u8]) {
    // RED GRN BL
    const WHITE_COLOUR: u8 = 0b_000_111_00;
    const BLACK_COLOUR: u8 = 0b_000_000_00;

    assert_eq!(vram.len(), 0x1BFF); // 2400 - 3FFF, 256x224 pixels - rotated 224x256?
                                    //let mut v = Vec::with_capacity(256*224);//[[0u8;256]; 224];
    let mut slice = [[0u8; 224]; 256];
    //unsafe{v.set_len(256*224)};
    let mut x = 0usize;
    let mut y = 255usize;
    for byte in vram.iter() {
        for pixel in get_bitvec(*byte).iter() {
            if *pixel {
                slice[y][x] = WHITE_COLOUR;
            } else {
                slice[y][x] = BLACK_COLOUR;
            }
            if y == 0 {
                x += 1;
                y = 255;
            } else {
                y -= 1;
            }
        }
    }
    let t: Vec<_> = slice.iter().flat_map(|x| x.to_vec()).collect();
    tex.with_lock(None, |buf, _pitch| {
        buf.copy_from_slice(&t);
    })
    .unwrap();
    //tex.update(None, &t, 224).unwrap();
    canvas.copy(&tex, None, None).unwrap();
}

pub fn main() {
    let mut emu = RS8080::new(Box::new(SpaceInvadersIO::new()));
    let h = include_bytes!("../../roms/invaders.h");
    let g = include_bytes!("../../roms/invaders.g");
    let f = include_bytes!("../../roms/invaders.f");
    let e = include_bytes!("../../roms/invaders.e");
    emu.load_to_mem(h, 0);
    emu.load_to_mem(g, 0x0800);
    emu.load_to_mem(f, 0x1000);
    emu.load_to_mem(e, 0x1800);
    emu.set_mem_limiter(Box::new(SpaceInvadersLimit {}));

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("rs8080-gui", 224 * 3, 256 * 3)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let tc = canvas.texture_creator();
    let mut texture = tc
        .create_texture_streaming(PixelFormatEnum::RGB332, 224, 256)
        .unwrap();
    let mut flipflop = false;
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        let start = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        draw_space_invaders_vram(&mut canvas, &mut texture, &emu.get_mem()[0x2400..0x3FFF]);
        canvas.present();

        // 2 MHz = 2 * 10^6 Hz; 500 ns -- 1 cycle; 1/60/(500*10^-9) = 33333.333
        //let start = Instant::now();
        let mut cycles_left = 33333i32;
        while cycles_left > 0 {
            let cycles = emu.emulate_next();
            cycles_left -= cycles.0 as i32;
        }
        if emu.int_enabled() {
            if flipflop {
                //emu.generate_interrupt(2);
                emu.call_interrupt(0x10);
            } else {
                //emu.generate_interrupt(1);
                emu.call_interrupt(0x8);
            }
            flipflop = !flipflop;
        }

        let elapsed = start.elapsed();
        
        //println!("elapsed: {:?}", elapsed);
        //thread::sleep(Duration::from_secs_f64(1f64 / 60f64));
        if elapsed > Duration::from_secs_f64(1f64 / 120f64){
            continue;
        }
        thread::sleep(Duration::from_secs_f64(1f64 / 120f64) - elapsed);
    }
}
