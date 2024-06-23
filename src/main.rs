#![allow(dead_code)]

use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};
use clap::Parser;
use macroquad::prelude::*;
use rfd::FileDialog;
use std::fs::{metadata, File};
use std::io::Read;
use std::path::PathBuf;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;
const PIXEL_SIZE: u32 = 10;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Optional ROM file
    filename: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PixelState {
    On,
    Off,
}

#[derive(Debug)]
struct Instruction {
    full: u16,
    op: u8,
    x: usize,
    y: usize,
    n: u8,
    nn: u8,
    nnn: u16,
}

impl Instruction {
    fn from(full: u16) -> Self {
        Self {
            full,
            op: (full >> 12) as u8,
            x: ((full >> 8) & 0xF) as usize,
            y: ((full >> 4) & 0xF) as usize,
            n: (full & 0xF) as u8,
            nn: (full & 0xFF) as u8,
            nnn: (full & 0xFFF),
        }
    }
}

struct Display {
    image: Image,
    texture: Texture2D,
}

impl Display {
    fn new() -> Self {
        let image = Image::gen_image_color(
            (WIDTH * PIXEL_SIZE) as u16,
            (HEIGHT * PIXEL_SIZE) as u16,
            BLACK,
        );
        Self {
            texture: Texture2D::from_image(&image),
            image,
        }
    }

    fn update(&mut self, pixels: &[PixelState]) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.update_image_rect(x, y, pixels[(y * WIDTH + x) as usize])
            }
        }
        self.texture.update(&self.image);
    }

    fn update_image_rect(&mut self, x: u32, y: u32, state: PixelState) {
        let start_x = x * PIXEL_SIZE;
        let start_y = y * PIXEL_SIZE;

        for x_offset in start_x..start_x + PIXEL_SIZE {
            if start_x + x_offset >= WIDTH * PIXEL_SIZE {
                continue;
            }

            for y_offset in start_y..start_y + PIXEL_SIZE {
                if start_y + y_offset >= HEIGHT * PIXEL_SIZE {
                    continue;
                }

                self.image.set_pixel(
                    start_x + x_offset,
                    start_y + y_offset,
                    match state {
                        PixelState::Off => BLACK,
                        PixelState::On => WHITE,
                    },
                )
            }
        }
    }
}

struct Chip8 {
    display: Display,
    pixels: Vec<PixelState>,
    memory: Vec<u8>,
    stack: Vec<usize>,
    v: Vec<u8>,
    i: u16,
    pc: usize,
    update_display: bool,
}

impl Chip8 {
    fn new() -> Self {
        let mut result = Self {
            display: Display::new(),
            pixels: vec![PixelState::Off; 2048],
            memory: vec![0; 4096],
            stack: vec![],
            v: vec![0; 16],
            i: 0,
            pc: 0x200,
            update_display: true,
        };
        result.memory[0x50..0x50 + FONT.len()].copy_from_slice(&FONT[..]);
        result
    }

    fn load_rom(&mut self, filename: &PathBuf) -> Result<()> {
        let mut f = File::open(filename)?;
        let metadata = metadata(filename)?;
        let mut rom = vec![0; metadata.len() as usize];
        f.read_exact(&mut rom)?;

        self.memory[self.pc..(rom.len() + self.pc)].copy_from_slice(&rom[..]);
        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            if is_key_pressed(KeyCode::Q) {
                return Ok(());
            }

            let instruction = self.decode();
            self.execute(instruction);

            if self.update_display {
                self.pixels[0] = PixelState::On;
                self.pixels[1] = PixelState::On;
                self.display.update(&self.pixels);
                self.update_display = false
            }

            draw_texture(&self.display.texture, 0., 0., GRAY);
            next_frame().await
        }
    }

    fn decode(&self) -> Instruction {
        Instruction::from(BigEndian::read_u16(&self.memory[self.pc..=self.pc + 1]))
    }

    fn execute(&mut self, ins: Instruction) {
        self.pc += 2;

        match ins.op {
            0x0 => match ins.nn {
                // clear screen
                0xE0 => {
                    self.clear_pixels();
                    self.update_display = true
                }

                // return from subroutine
                0xEE => self.pc = self.stack.pop().unwrap(),

                // default
                _ => (),
            },

            // jump to nnn
            0x1 => self.pc = ins.nnn as usize,

            // call subroutine nnn
            0x2 => {
                self.stack.push(self.pc);
                self.pc = ins.nnn as usize
            }

            // pc += 2 if Vx == nn
            0x3 if self.v[ins.x] == ins.nn => self.pc += 2,

            // pc += 2 if Vx != nn
            0x4 if self.v[ins.x] != ins.nn => self.pc += 2,

            // pc += 2 if Vx == Vy
            0x5 if self.v[ins.x] == self.v[ins.y] => self.pc += 2,

            // Vx = nn
            0x6 => self.v[ins.x] = ins.nn,

            // Vx += nn (wrapping)
            0x7 => self.v[ins.x] = self.v[ins.x].wrapping_add(ins.nn),

            // Vx operations
            0x8 => match ins.n {
                // Vx = Vy
                0x0 => self.v[ins.x] = self.v[ins.y],

                // Vx = Vx | Vy
                0x1 => self.v[ins.x] |= self.v[ins.y],

                // Vx = Vx & Vy
                0x2 => self.v[ins.x] &= self.v[ins.y],

                // Vx = Vx ^ Vy
                0x3 => self.v[ins.x] ^= self.v[ins.y],

                // Vx = Vx + Vy
                0x4 => {
                    let mut res = self.v[ins.x] as u16 + self.v[ins.y] as u16;
                    if res > 255 {
                        res -= 256;
                        self.v[0xF] = 1
                    } else {
                        self.v[0xF] = 0
                    }
                    self.v[ins.x] = res as u8
                }

                0x5..=0x7 => todo!(),

                // default
                _ => (),
            },

            // pc += 2 if Vx != Vy
            0x9 if self.v[ins.x] != self.v[ins.y] => self.pc += 2,

            // I = nnn
            0xA => self.i = ins.nnn,

            // pc = nnn + V0
            0xB => self.pc = (ins.nnn + self.v[0] as u16) as usize,

            // Vx = rand(0-255) & nn
            0xC => self.v[ins.x] = rand::gen_range(0u8, 255u8) & ins.nn,

            // draw things :3
            0xD => {
                self.draw_sprite(ins);
                self.update_display = true;
            }

            0xE => todo!(), // keypress things

            0xF => todo!(), // lots of random shit

            // default, do nothing
            _ => (),
        }
    }

    fn clear_pixels(&mut self) {
        for pixel in self.pixels.iter_mut() {
            *pixel = PixelState::Off
        }
    }

    fn draw_sprite(&mut self, ins: Instruction) {
        // start coords, wrapped round (modulo screen dimensions)
        let start_x = self.v[ins.x] & 63;
        let start_y = self.v[ins.y] & 31;
        let height = ins.n;
        self.v[0xF] = 0;

        for row in 0..height {
            let sprite_row = self.memory[self.i as usize + row as usize];

            for col in 0..8u8 {
                let sprite_pixel = match sprite_row & (0x80 >> col) {
                    0 => PixelState::Off,
                    _ => PixelState::On,
                };
                let screen_pixel = &mut self.pixels
                    [((start_y + row) as u32 * WIDTH + (start_x + col) as u32) as usize];

                // if pixel is on in the sprite
                if sprite_pixel == PixelState::On {
                    match (&sprite_pixel, &screen_pixel) {
                        (&PixelState::On, &PixelState::On) => {
                            *screen_pixel = PixelState::Off;
                            self.v[0xF] = 1
                        }
                        (&PixelState::On, &PixelState::Off) => *screen_pixel = PixelState::On,
                        (&PixelState::Off, &PixelState::On) => *screen_pixel = PixelState::On,
                        (&PixelState::Off, &PixelState::Off) => *screen_pixel = PixelState::Off,
                    }
                }
            }
        }
    }
}

fn conf() -> Conf {
    Conf {
        window_title: String::from("Chip8 Emulator"),
        window_width: WIDTH as i32 * PIXEL_SIZE as i32,
        window_height: HEIGHT as i32 * PIXEL_SIZE as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() -> Result<()> {
    let args = Args::parse();

    let file = match args.filename {
        Some(file) => file,
        None => match FileDialog::new().add_filter("Rom", &["ch8"]).pick_file() {
            Some(file) => file,
            None => PathBuf::new(),
        },
    };

    let mut chip = Chip8::new();
    chip.load_rom(&file)?;
    chip.run().await?;

    Ok(())
}
