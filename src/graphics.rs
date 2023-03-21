use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
    EventPump,
};

use crate::{
    palette,
    ppu::{ControlRegister, Ppu},
    util::Error,
};

const TILE_LENGTH: u16 = 8;
const PATTERN_TABLE_TILE_LENGTH: u16 = TILE_LENGTH * 2;
const SCREEN_WIDTH: u16 = 256;
const SCREEN_HEIGHT: u16 = 240;
const SCREEN_WIDTH_TILES: u16 = SCREEN_WIDTH / TILE_LENGTH;
const SCREEN_HEIGHT_TILES: u16 = SCREEN_HEIGHT / TILE_LENGTH;

const PIXEL_MULTIPLIER: u16 = 2;

pub struct Frame {
    pub data: Vec<u8>,
}

impl Frame {
    const WIDTH: usize = 256;
    const HIGHT: usize = 240;

    pub fn new() -> Self {
        Frame {
            data: vec![0; (Frame::WIDTH) * (Frame::HIGHT) * 3],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let base = y * 3 * Frame::WIDTH + x * 3;
        if base + 2 < self.data.len() {
            self.data[base] = rgb.0;
            self.data[base + 1] = rgb.1;
            self.data[base + 2] = rgb.2;
        }
    }
}

// pub fn show_tiles(chr_rom: &Vec<u8>) -> Frame {
//     let mut frame = Frame::new();
//     // Pixel width = 8 *
//     fn show_bank(chr_rom: &Vec<u8>, frame: &mut Frame, bank_1: bool) {
//         let tiles: u16 = chr_rom.len() as u16 / (TILE_LENGTH * 2);
//         let bank_offset = if bank_1 { 0x1000 } else { 0 };
//         let y_offset = if bank_1 {
//             TILE_LENGTH * tiles / 0x20
//         } else {
//             0
//         };
//         for tile_n in 0..tiles {
//             let tile_start = bank_offset + tile_n * PATTERN_TABLE_TILE_LENGTH;
//             let tile_end =
//                 bank_offset + tile_n * PATTERN_TABLE_TILE_LENGTH + PATTERN_TABLE_TILE_LENGTH - 1;
//             let tile = &chr_rom[tile_start as usize..=tile_end as usize];

//             let tile_col = tile_n % SCREEN_WIDTH_TILES;
//             let tile_row = tile_n / SCREEN_WIDTH_TILES;
//             let tile_y = tile_row * TILE_LENGTH;
//             let tile_x = tile_col * TILE_LENGTH;

//             for y in 0..TILE_LENGTH {
//                 let mut upper = tile[y as usize];
//                 let mut lower = tile[(y + TILE_LENGTH) as usize];

//                 for x in (0..TILE_LENGTH).rev() {
//                     let value = (1 & upper) << 1 | (1 & lower);
//                     upper = upper >> 1;
//                     lower = lower >> 1;
//                     let rgb = match value {
//                         0 => palette::SYSTEM_PALLETE[0x01],
//                         1 => palette::SYSTEM_PALLETE[0x23],
//                         2 => palette::SYSTEM_PALLETE[0x27],
//                         3 => palette::SYSTEM_PALLETE[0x30],
//                         _ => panic!("can't be"),
//                     };
//                     frame.set_pixel(tile_x + x, y_offset + tile_y + y, rgb)
//                 }
//             }
//         }
//     }

//     // show_bank(chr_rom, &mut frame, false);
//     show_bank(chr_rom, &mut frame, true);

//     frame
// }

pub struct Graphics {
    frame: Frame,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    event_pump: EventPump,
}

impl Graphics {
    pub fn new() -> Result<Self, Error> {
        let frame = Frame::new();

        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window_width = (SCREEN_WIDTH * PIXEL_MULTIPLIER) as u32;
        let window_height = (SCREEN_HEIGHT * PIXEL_MULTIPLIER) as u32;
        let window = video_subsystem
            .window("NES", window_width, window_height)
            .position_centered()
            .build()?;

        let canvas = window.into_canvas().present_vsync().build()?;
        let event_pump = sdl_context.event_pump()?;

        let texture_creator = canvas.texture_creator();

        Ok(Graphics {
            frame,
            canvas,
            texture_creator,
            event_pump,
        })
    }

    pub fn render(&mut self, ppu: &Ppu) -> Result<(), Error> {
        render_to_frame(ppu, &mut self.frame);
        let mut texture = self.texture_creator.create_texture_target(
            PixelFormatEnum::RGB24,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        )?;
        texture.update(None, &self.frame.data, 256 * 3)?;
        self.canvas.copy(&texture, None, None)?;
        self.canvas.present();

        loop {
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),
                    _ => { /* do nothing */ }
                }
            }
        }
    }
}

/**
 * Mutates the given frame, rendering the PPU's output to it
 */
fn render_to_frame(ppu: &Ppu, frame: &mut Frame) {
    let pattern_table_offset = ppu.control.background_pattern_offset();

    for tile_n in 0..(SCREEN_WIDTH_TILES * SCREEN_HEIGHT_TILES) {
        // Get the nth tile's pattern table index from the nametable
        let tile_pattern_n = ppu.vram[tile_n as usize] as u16;
        let tile_x = tile_n % SCREEN_WIDTH_TILES;
        let tile_y = tile_n / SCREEN_WIDTH_TILES;

        let tile_pattern_data_start =
            pattern_table_offset + tile_pattern_n * PATTERN_TABLE_TILE_LENGTH;
        let tile_pattern_data_end = pattern_table_offset
            + tile_pattern_n * PATTERN_TABLE_TILE_LENGTH
            + PATTERN_TABLE_TILE_LENGTH
            - 1;
        let tile_pattern_data =
            &ppu.chr_rom[tile_pattern_data_start as usize..=tile_pattern_data_end as usize];

        // x and y are relative within tile_n
        for y in 0..TILE_LENGTH {
            let mut left_bit_row = tile_pattern_data[y as usize];
            let mut right_bit_row = tile_pattern_data[y as usize + 8];

            for x in (0..TILE_LENGTH).rev() {
                let pixel_value = (left_bit_row & 0x01) << 1 | (right_bit_row & 0x01);
                left_bit_row = left_bit_row >> 1;
                right_bit_row = right_bit_row >> 1;
                let rgb = match pixel_value {
                    0 => palette::SYSTEM_PALLETE[0x01],
                    1 => palette::SYSTEM_PALLETE[0x23],
                    2 => palette::SYSTEM_PALLETE[0x27],
                    3 => palette::SYSTEM_PALLETE[0x30],
                    _ => panic!("can't be"),
                };
                let pixel_x = tile_x * TILE_LENGTH + x;
                let pixel_y = tile_y * TILE_LENGTH + y;
                frame.set_pixel(pixel_x as usize, pixel_y as usize, rgb);
            }
        }
    }
}
