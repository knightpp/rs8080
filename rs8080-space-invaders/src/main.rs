extern crate sdl2;
extern crate rs8080_emulator as emulator;
use emulator::{RS8080, DataBus};
use emulator::{MemLimiter, WriteAction};

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::time::{Instant, Duration};
use sdl2::rect::{Point, Rect};
use sdl2::render::{WindowCanvas, Texture};
use std::thread;

fn get_bitvec(byte :u8) -> [bool; 8]{
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

struct SpaceInvadersLimit{
}
impl MemLimiter for SpaceInvadersLimit{
    fn check_write(&self, adr : u16, _ : u8) -> WriteAction { 
        if adr < 0x2000{
            eprintln!("block: write mem < 0x2000");
            WriteAction::Ignore
        }else if adr >= 0x4000{
            eprintln!("block: write mem >= 0x4000");
            WriteAction::Ignore
        }else{
            WriteAction::Allow
        }
     }
    fn check_read(&self, _ : u16, read_byte : u8) -> u8 { read_byte }

}


pub struct SpaceInvadersIO{
    ports: [u8; 6],
    shift0: u8,
    shift1: u8,
    shift_offset: u8,
}

impl SpaceInvadersIO{
    pub fn new() -> SpaceInvadersIO{
        SpaceInvadersIO{
            ports: [0; 6],
            shift0: 0,
            shift1: 0,
            shift_offset: 0
        }
    }
}

impl DataBus for SpaceInvadersIO{

    fn port_in(&mut self, port: u8) -> u8 {
        //panic!("port_in");
       // if port != 1{
          //  println!("port_in: port={}", port);
       // }
        match port {
            3 => {
                //println!("actually shif data");
                let v: u16 = ((self.shift1 as u16) << 8) | self.shift0 as u16;
                ((v >> (8u8 - self.shift_offset) as u16) & 0xFF) as u8
            }
            _ => {0}
        }

    }

    fn port_out(&mut self, value: u8, port: u8) {
       // if port != 6{
          //  println!("port_out: {}, value: {}", port, value);
       // }
        match port {
            2 => {
                //println!("set shift amount");
                self.shift_offset = value & 0b0000_0111u8;
            }
            4 => {
                //println!("set next shift");
                self.shift0 = self.shift1;
                self.shift1 = value;
            }
            _ => {}
        }
    }

    fn port(&mut self, index : usize) -> &mut u8{
        &mut self.ports[index]
    }
}


pub fn draw_space_invaders_vram(canvas: &mut WindowCanvas,tex :&mut Texture,  vram : &[u8]){
                              // RED GRN BL
    const WHITE_COLOUR : u8 = 0b_000_111_00;
    const BLACK_COLOUR : u8 = 0b_000_000_00;

    assert_eq!(vram.len(), 0x1BFF); // 2400 - 3FFF, 256x224 pixels - rotated 224x256?
    //let mut v = Vec::with_capacity(256*224);//[[0u8;256]; 224];
    let mut slice = [[0u8;224]; 256];
    //unsafe{v.set_len(256*224)};
    let mut x = 0usize;
    let mut y = 255usize;
    for byte in vram.iter(){
        for  pixel in get_bitvec(*byte).iter(){
            if *pixel{
                slice[y][x] = WHITE_COLOUR;
            }else{
                slice[y][x] = BLACK_COLOUR;
            } 
            if y == 0 {
                x += 1;
                y = 255;
            }else{
                y -= 1;
            }
        }
    }

    // let data = vram.iter().flat_map(|x|{
    //     let bits = get_bitvec(*x);
    //     bits.iter().map(|bit| if *bit {0xffu8}else{0u8}).collect::<Vec<_>>()
    // }).collect::<Vec<_>>();
  

    //  tex.with_lock(None, |buf, pitch|{
    //     buf.copy_from_slice(&data[..buf.len()]);
    //  }).unwrap();

    let t : Vec<_> = slice.iter().flat_map(|x| x.to_vec()).collect();
    tex.update(None, &t, 224).unwrap();
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

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rs8080-gui", 224*3, 256*3)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let tc = canvas.texture_creator();
    let mut texture = tc.create_texture_streaming(PixelFormatEnum::RGB332, 224, 256).unwrap();
    let mut time_since_last_int = Instant::now();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        //for _ in event_pump.poll_iter(){}
        
       // println!("{}", emu.disassemble_next());
        let now = Instant::now();
        if now.duration_since(time_since_last_int) > Duration::from_secs_f64(1f64/60f64){
            
            for event in event_pump.poll_iter() {
                match event {
                    Event::KeyDown{keycode : Some(Keycode::X) ,..} => {
                        //emu.generate_int(0x10);
                       // println!("{:?}", emu.get_io_mut().ports());
                        //*emu.get_io_mut().port(1) |= 0x1;
                        println!("X down");
                    }
                    Event::KeyDown{keycode : Some(Keycode::C) ,..} => {
                        //emu.generate_int(0x8);
                       // println!("{:?}", emu.get_io_mut().ports());
                       *emu.get_io_mut().port(1) |= 0x1;
                        println!("C down");
                    }
                    Event::KeyUp{keycode : Some(Keycode::C) ,..} => {
                       // println!("{:?}", emu.get_io_mut().ports());
                       *emu.get_io_mut().port(1)  &= !0x1;
                      
                        println!("C up");
                    }
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    _ => {}
                }
            }
            draw_space_invaders_vram(&mut canvas,&mut texture,emu.get_mem_slice(0x2400..0x3FFF));
            canvas.present();
            if emu.int_enabled(){
                //emu.generate_interrupt(2);
                emu.generate_int(0x8);
                emu.generate_int(0x10);
                //emu.emulate_next();
            }
            emu.emulate_next();
            time_since_last_int = now;
        }else{
            emu.emulate_next();
        }

        

        // for event in event_pump.wait_iter(){
        //     if let Event::MouseButtonDown{..} = event{
        //         for _ in 0..100{
        //             println!("{} {}", emu.disassemble_next(), emu);
        //             emu.emulate_next();
        //         }
        //        // break;
        //     }
        // }
    }
}
