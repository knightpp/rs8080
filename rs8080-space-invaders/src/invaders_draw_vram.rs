use sdl2::render::{Texture, WindowCanvas};

pub(crate) fn draw_space_invaders_vram(
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
