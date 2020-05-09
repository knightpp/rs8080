extern crate rs8080_emulator as emulator;
extern crate sdl2;
use emulator::{DataBus, RS8080};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::{pixels::PixelFormatEnum, video::FullscreenType};
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::{
    thread,
    time::{Duration, Instant},
};

mod config;
mod invaders_draw_vram;
mod invaders_io;
mod invaders_mem_limit;
#[cfg(feature = "sound")]
mod invaders_sound;

use config::{load_config, Config};
use invaders_draw_vram::draw_space_invaders_vram;
use invaders_io::SpaceInvadersIO;
use invaders_mem_limit::SpaceInvadersLimit;

// Replace Ok with never type '!'
fn run_space_invaders_machine(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let keycodes = config.controls;
    let io = SpaceInvadersIO::new();
    #[cfg(feature = "sound")]
    {
        io.get_audio().set_volume(config.volume.volume);
    }
    let mut emu = RS8080::new_with_limit(io, SpaceInvadersLimit {});

    #[cfg(feature = "bundlerom")]
    {
        let h = include_bytes!("../../roms/invaders.h");
        let g = include_bytes!("../../roms/invaders.g");
        let f = include_bytes!("../../roms/invaders.f");
        let e = include_bytes!("../../roms/invaders.e");
        emu.load_to_mem(h, 0);
        emu.load_to_mem(g, 0x0800);
        emu.load_to_mem(f, 0x1000);
        emu.load_to_mem(e, 0x1800);
    }
    #[cfg(not(feature = "bundlerom"))]
    {
        use std::io::ErrorKind;
        let h_err =
            std::io::Error::new(std::io::ErrorKind::Other, "cannot read './roms/invaders.h'");
        let g_err =
            std::io::Error::new(std::io::ErrorKind::Other, "cannot read './roms/invaders.g'");
        let f_err =
            std::io::Error::new(std::io::ErrorKind::Other, "cannot read './roms/invaders.f'");
        let e_err =
            std::io::Error::new(std::io::ErrorKind::Other, "cannot read './roms/invaders.e'");

        let h = std::fs::read("./roms/invaders.h").or(Err(h_err))?;
        let g = std::fs::read("./roms/invaders.g").or(Err(g_err))?;
        let f = std::fs::read("./roms/invaders.f").or(Err(f_err))?;
        let e = std::fs::read("./roms/invaders.e").or(Err(e_err))?;
        emu.load_to_mem(&h, 0);
        emu.load_to_mem(&g, 0x0800);
        emu.load_to_mem(&f, 0x1000);
        emu.load_to_mem(&e, 0x1800);
    }
    
    let sdl_context = sdl2::init()?;
    let _audio = sdl_context.audio()?;
    let video_subsystem = sdl_context.video()?;
    
    let mut window = video_subsystem
        .window(
            "rs8080-space-invaders",
            config.screen.width,
            config.screen.height,
        )
        .position_centered()
        .resizable()
        .build()?;
    if config.screen.fullscreen {
        window.set_fullscreen(FullscreenType::Desktop).unwrap();
    }

    let mut canvas = window.into_canvas().present_vsync().accelerated().build()?;
    let tc = canvas.texture_creator();
    let mut texture = tc.create_texture_streaming(PixelFormatEnum::RGB332, 224, 256)?;
    let mut flipflop = false;
    let mut event_pump = sdl_context.event_pump()?;

    //let mut fps = 0u64;
    //let fps_start = Instant::now();
    'running: loop {
        let start = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(..),
                    ..
                } => {
                    canvas.clear();
                }
                Event::KeyDown {
                    keycode: Some(x), ..
                } => {
                    if keycodes.insert_coin.eq(&x) {
                        *emu.get_io_mut().port(1) |= 0x1;
                    }
                    if keycodes.start_1p.eq(&x) {
                        *emu.get_io_mut().port(1) |= 0b0000_0100;
                    }
                    if keycodes.start_2p.eq(&x) {
                        *emu.get_io_mut().port(1) |= 0b0000_0010;
                    }
                    if keycodes.shot_1p.eq(&x) {
                        *emu.get_io_mut().port(1) |= 0b0001_0000;
                    }
                    if keycodes.shot_2p.eq(&x) {
                        *emu.get_io_mut().port(2) |= 0b0001_0000;
                    }
                    if keycodes.left_1p.eq(&x) {
                        *emu.get_io_mut().port(1) |= 0b0010_0000;
                    }
                    if keycodes.left_2p.eq(&x) {
                        *emu.get_io_mut().port(2) |= 0b0010_0000;
                    }
                    if keycodes.right_1p.eq(&x) {
                        *emu.get_io_mut().port(1) |= 0b0100_0000;
                    }
                    if keycodes.right_2p.eq(&x) {
                        *emu.get_io_mut().port(2) |= 0b0100_0000;
                    }

                    if Keycode::Escape.eq(&x) {
                        break 'running;
                    }
                }
                Event::KeyUp {
                    keycode: Some(x), ..
                } => {
                    if keycodes.insert_coin.deref().eq(&x) {
                        *emu.get_io_mut().port(1) &= !0x1;
                    }
                    if keycodes.start_1p.eq(&x) {
                        *emu.get_io_mut().port(1) &= !0b0000_0100;
                    }
                    if keycodes.start_2p.eq(&x) {
                        *emu.get_io_mut().port(1) &= !0b0000_0010;
                    }
                    if keycodes.shot_1p.eq(&x) {
                        *emu.get_io_mut().port(1) &= !0b0001_0000;
                    }
                    if keycodes.shot_2p.eq(&x) {
                        *emu.get_io_mut().port(2) &= !0b0001_0000;
                    }
                    if keycodes.left_1p.eq(&x) {
                        *emu.get_io_mut().port(1) &= !0b0010_0000;
                    }
                    if keycodes.left_2p.eq(&x) {
                        *emu.get_io_mut().port(2) &= !0b0010_0000;
                    }
                    if keycodes.right_1p.eq(&x) {
                        *emu.get_io_mut().port(1) &= !0b0100_0000;
                    }
                    if keycodes.right_2p.eq(&x) {
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
                if flipflop {
                    emu.call_interrupt(0x10);
                } else {
                    emu.call_interrupt(0x8);
                }
                flipflop = !flipflop;
            }
        }
        draw_space_invaders_vram(
            &mut canvas,
            &mut texture,
            &emu.get_mem()[0x2400..0x3FFF],
            &config.screen,
        )?;
        canvas.present();
        let elapsed = start.elapsed();
        //println!("End of frame reached after {:?} ms", elapsed.as_millis());
        if !(elapsed > Duration::from_secs_f64(1f64 / 60f64)) {
            thread::sleep(Duration::from_secs_f64( 1f64 / 60f64) - elapsed);
        }
        //println!("fps = {}", fps as f64/fps_start.elapsed().as_secs_f64());
        //fps += 1;
    }
    
    Ok(())
}

macro_rules! handle_err {
    ($($args:tt)+) => {
        match $($args)+{
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
        handle_err!(handle_err!(File::create("config.toml")).write_all(default_config));
    }
    let config = handle_err!(load_config("config.toml"));
    handle_err!(run_space_invaders_machine(config));
}
