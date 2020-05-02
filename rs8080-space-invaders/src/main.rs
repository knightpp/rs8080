extern crate rs8080_emulator as emulator;
extern crate sdl2;
use emulator::{DataBus, RS8080};
use emulator::{MemLimiter, WriteAction};

use serde::{Deserialize};
use std::fs::File;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, WindowCanvas};
use serde::export::Formatter;
use std::io::{Read, Write};
use std::{
    fmt, thread,
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

    pub fn set_shift_offset(&mut self, offset: u8) {
        self.ports[2] = offset & 0x7;
    }
}

impl DataBus for SpaceInvadersIO {
    fn port_in(&mut self, port: u8) -> u8 {
        match port {
            0 => 0xf,
            1 => self.ports[1],
            3 => (((self.shift0 as u16) << 8) | self.shift1 as u16)
                .rotate_left(self.shift_offset as u32) as u8,
            _ => self.ports[port as usize],
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
mod tests {
    use super::*;

    #[test]
    fn invaders_shift_register() {
        let mut io = SpaceInvadersIO::new();
        io.port_out(0xFF, 4); // write 0xFF to shift1
        assert_eq!(0xFF, io.shift1);
        io.port_out(0, 2); // set offset to 0
        assert_eq!(io.shift_offset, 0);
        io.port_out(0b0000_0111, 2); // set shift_offset to 7
        assert_eq!(io.shift_offset, 7);
        assert_eq!(io.port_in(3), 0xFF << 7);

        io.port_out(13, 4); // write 13 to shift1, shift0 = 0xFF
        io.port_out(3, 2); // set shift_offset to 3
        assert_eq!(io.port_in(3), (0x0DFF >> (8 - 3)) as u8);
    }
}

pub fn draw_space_invaders_vram(
    canvas: &mut WindowCanvas,
    tex: &mut Texture,
    vram: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
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
    })?;
    //tex.update(None, &t, 224).unwrap();
    canvas.copy(&tex, None, None)?;
    Ok(())
}

#[derive(Deserialize)]
struct Controls {
    insert_coin: String,
    start_2p: String,
    start_1p: String,
    shot_1p: String,
    shot_2p: String,
    left_1p: String,
    left_2p: String,
    right_1p: String,
    right_2p: String,
}

#[derive(Debug)]
struct Keycodes {
    insert_coin: Keycode,
    start_1p: Keycode,
    start_2p: Keycode,
    shot_1p: Keycode,
    shot_2p: Keycode,
    left_1p: Keycode,
    left_2p: Keycode,
    right_1p: Keycode,
    right_2p: Keycode,
}

#[derive(Deserialize)]
struct Config {
    controls: Controls,
}

fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

// Replace Ok with never type '!'
fn run_space_invaders_machine(keycodes: Keycodes) -> Result<(), Box<dyn std::error::Error>> {
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

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rs8080-gui", 224 * 3, 256 * 3)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let tc = canvas.texture_creator();
    let mut texture = tc.create_texture_streaming(PixelFormatEnum::RGB332, 224, 256)?;
    let mut flipflop = false;
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        let start = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(x), ..
                } => {
                    if x == keycodes.insert_coin {
                        *emu.get_io_mut().port(1) |= 0x1;
                    }
                    if x == keycodes.start_1p {
                        *emu.get_io_mut().port(1) |= 0b0000_0100;
                    }
                    if x == keycodes.start_2p {
                        *emu.get_io_mut().port(1) |= 0b0000_0010;
                    }
                    if x == keycodes.shot_1p {
                        *emu.get_io_mut().port(1) |= 0b0001_0000;
                    }
                    if x == keycodes.shot_2p {
                        *emu.get_io_mut().port(2) |= 0b0001_0000;
                    }
                    if x == keycodes.left_1p {
                        *emu.get_io_mut().port(1) |= 0b0010_0000;
                    }
                    if x == keycodes.left_2p {
                        *emu.get_io_mut().port(2) |= 0b0010_0000;
                    }
                    if x == keycodes.right_1p {
                        *emu.get_io_mut().port(1) |= 0b0100_0000;
                    }
                    if x == keycodes.right_2p {
                        *emu.get_io_mut().port(2) |= 0b0100_0000;
                    }

                    if x == Keycode::Escape {
                        break 'running;
                    }
                }
                Event::KeyUp {
                    keycode: Some(x), ..
                } => {
                    if x == keycodes.insert_coin {
                        *emu.get_io_mut().port(1) &= !0x1;
                    }
                    if x == keycodes.start_1p {
                        *emu.get_io_mut().port(1) &= !0b0000_0100;
                    }
                    if x == keycodes.start_2p {
                        *emu.get_io_mut().port(1) &= !0b0000_0010;
                    }
                    if x == keycodes.shot_1p {
                        *emu.get_io_mut().port(1) &= !0b0001_0000;
                    }
                    if x == keycodes.shot_2p {
                        *emu.get_io_mut().port(2) &= !0b0001_0000;
                    }
                    if x == keycodes.left_1p {
                        *emu.get_io_mut().port(1) &= !0b0010_0000;
                    }
                    if x == keycodes.left_2p {
                        *emu.get_io_mut().port(2) &= !0b0010_0000;
                    }
                    if x == keycodes.right_1p {
                        *emu.get_io_mut().port(1) &= !0b0100_0000;
                    }
                    if x == keycodes.right_2p {
                        *emu.get_io_mut().port(2) &= !0b0100_0000;
                    }
                }
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        // 2 MHz = 2 * 10^6 Hz; 500 ns -- 1 cycle; 1/60/(500*10^-9) = 33333.333
        for _ in 0..2 {
            let mut cycles_left = 33333i32 / 2;
            while cycles_left > 0 {
                let cycles = emu.emulate_next();
                cycles_left -= cycles.0 as i32;
            }
            if emu.int_enabled() {
                draw_space_invaders_vram(
                    &mut canvas,
                    &mut texture,
                    &emu.get_mem()[0x2400..0x3FFF],
                )?;
                canvas.present();
                if flipflop {
                    emu.call_interrupt(0x10);
                } else {
                    emu.call_interrupt(0x8);
                }
                flipflop = !flipflop;
            }
        }

        let elapsed = start.elapsed();
        //println!("elapsed: {:?}", elapsed);
        //thread::sleep(Duration::from_secs_f64(1f64 / 60f64));
        if elapsed > Duration::from_secs_f64(1f64 / 60f64) {
            continue;
        }
        thread::sleep(Duration::from_secs_f64(1f64 / 60f64) - elapsed);
    }

    Ok(())
}

#[derive(Copy, Clone)]
struct ParseKeycodeErr {}
impl std::fmt::Display for ParseKeycodeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", "failed to parse keycode from string")
    }
}
impl std::fmt::Debug for ParseKeycodeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", "failed to parse keycode from string")
    }
}
impl std::error::Error for ParseKeycodeErr {}

fn parse_keycodes(controls: Controls) -> Result<Keycodes, Box<dyn std::error::Error>> {
    let err = ParseKeycodeErr {};
    Ok(Keycodes {
        insert_coin: Keycode::from_name(&controls.insert_coin).ok_or(err)?,
        start_1p: Keycode::from_name(&controls.start_1p).ok_or(err)?,
        start_2p: Keycode::from_name(&controls.start_2p).ok_or(err)?,
        shot_1p: Keycode::from_name(&controls.shot_1p).ok_or(err)?,
        shot_2p: Keycode::from_name(&controls.shot_2p).ok_or(err)?,
        left_1p: Keycode::from_name(&controls.left_1p).ok_or(err)?,
        left_2p: Keycode::from_name(&controls.left_2p).ok_or(err)?,
        right_1p: Keycode::from_name(&controls.right_1p).ok_or(err)?,
        right_2p: Keycode::from_name(&controls.right_2p).ok_or(err)?,
    })
}

macro_rules! handle_err {
    ($f:expr, $($params:tt)*) => {
        match $f($($params)*,){
            Ok(x) => { x },
            Err(err) => {
                eprintln!("{}", err);
                if cfg!(target_os = "windows") {
                    use std::process::Command;
                    let _ = Command::new("cmd.exe").arg("/c").arg("pause").status();
                }
                std::process::exit(-1);
            },
        }
    };
}

fn main() {
    let default_config = include_bytes!("../../config.default.toml");

    if !std::path::Path::new("config.toml").exists() {
        File::create("config.toml")
            .unwrap()
            .write_all(default_config)
            .unwrap();
    }
    let config = handle_err!(load_config, "config.toml");
    let keycodes = handle_err!(parse_keycodes, config.controls);
    handle_err!(run_space_invaders_machine, keycodes);
}
