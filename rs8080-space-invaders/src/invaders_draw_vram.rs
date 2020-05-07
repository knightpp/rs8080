use crate::config::Screen;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};

pub(crate) fn draw_space_invaders_vram(
    canvas: &mut WindowCanvas,
    tex: &mut Texture,
    vram: &[u8],
    screen: &Screen,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(vram.len(), 0x1BFF);
    let mut slice = [[0u8; 224]; 256];
    let mut x = 0usize;
    let mut y = 255usize;
    for byte in vram.iter() {
        for pixel in get_bitvec(*byte).iter() {
            if *pixel {
                slice[y][x] = screen.white_color;
            } else {
                slice[y][x] = screen.black_color;
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
    let canvas_size = canvas.window().size();
    let new_width = (screen.width as f64 * canvas_size.1 as f64 / screen.height as f64) as u32;
    canvas.copy(
        &tex,
        None,
        Rect::new(
            canvas_size.0 as i32 / 2 - new_width as i32 / 2,
            0,
            new_width,
            canvas_size.1,
        ),
    )?;
    Ok(())
}

pub(crate) fn get_bitvec(byte: u8) -> [bool; 8] {
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
