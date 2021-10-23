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
    //let mut slice = [[0u8; 224]; 256];
    let mut x = 0usize;
    let mut y = 255usize;
    tex.with_lock(None, |buf, _pitch| {
        for byte in vram {
            for pixel_emit_light in ByteBitIter::new(*byte) {
                if pixel_emit_light {
                    buf[y * 224 + x] = screen.white_color;
                } else {
                    buf[y * 224 + x] = screen.black_color;
                }
                if y == 0 {
                    x += 1;
                    y = 255;
                } else {
                    y -= 1;
                }
            }
        }
    })?;

    //tex.update(None, &t, 224).unwrap();
    let canvas_size = canvas.window().size();
    let new_width = (screen.width as f64 * canvas_size.1 as f64 / screen.height as f64) as u32;
    canvas.copy(
        tex,
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

struct ByteBitIter {
    byte: u8,
    next_bit_pos: u8,
}
impl ByteBitIter {
    fn new(byte: u8) -> Self {
        ByteBitIter {
            byte,
            next_bit_pos: 0,
        }
    }
}

impl Iterator for ByteBitIter {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_bit_pos < 8 {
            let current_bit_pos = self.next_bit_pos;
            self.next_bit_pos += 1;
            Some(self.byte & (1 << current_bit_pos) > 0)
        } else {
            None
        }
    }
}
