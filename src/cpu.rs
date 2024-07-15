#![allow(dead_code)]

use rand::{rngs::ThreadRng, Rng};
use std::fs::{metadata, File};
use std::io::Read;
use std::path::PathBuf;

use byteorder::{BigEndian, ByteOrder};

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

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

pub struct Cpu {
    pub pixels: Vec<bool>,
    memory: Vec<u8>,
    stack: Vec<usize>,
    v: Vec<u8>,
    i: u16,
    pc: usize,
    delay_timer: u8,
    sound_timer: u8,
    rng: ThreadRng,
    pub update_display: bool,
}

impl Cpu {
    pub fn new() -> Self {
        let mut res = Self {
            pixels: vec![false; (SCREEN_HEIGHT * SCREEN_WIDTH) as usize],
            memory: vec![0; 4096],
            stack: vec![],
            v: vec![0; 16],
            i: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            rng: rand::thread_rng(),
            update_display: false,
        };
        res.memory[0x50..0x50 + FONT.len()].copy_from_slice(&FONT[..]);
        res
    }

    pub fn load_rom(&mut self, filename: &PathBuf) {
        let mut f = File::open(filename).expect("failed to open file");
        let metadata = metadata(filename).expect("failed to parse metadata");
        let mut rom = vec![0; metadata.len() as usize];
        f.read_exact(&mut rom).expect("failed to read");

        self.memory[self.pc..(rom.len() + self.pc)].copy_from_slice(&rom[..]);
    }

    pub fn tick(&mut self, keys: &[bool; 16]) {
        let instruction = self.decode();
        self.execute(instruction, keys)
    }

    fn decode(&self) -> Instruction {
        Instruction::from(BigEndian::read_u16(&self.memory[self.pc..=self.pc + 1]))
    }

    fn execute(&mut self, ins: Instruction, keys: &[bool; 16]) {
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

                _ => unimplemented!(),
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
            0x5 if ins.n == 0 => {
                if self.v[ins.x] == self.v[ins.y] {
                    self.pc += 2
                }
            }

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

                0xE => todo!(),

                _ => unimplemented!(),
            },

            // pc += 2 if Vx != Vy
            0x9 if ins.n == 0 => {
                if self.v[ins.x] != self.v[ins.y] {
                    self.pc += 2
                }
            }

            // I = nnn
            0xA => self.i = ins.nnn,

            // pc = nnn + V0
            0xB => self.pc = (ins.nnn + self.v[0] as u16) as usize,

            // Vx = rand(0-255) & nn
            0xC => self.v[ins.x] = self.rng.gen_range(0..255u8) & ins.nn,

            // draw things :3
            0xD => {
                self.draw_sprite(ins);
                self.update_display = true;
            }

            // keypress things :3
            0xE => match ins.nn {
                0x9E => {
                    if keys[self.v[ins.x] as usize] {
                        self.pc += 2
                    }
                }
                0xA1 => {
                    if !keys[self.v[ins.x] as usize] {
                        self.pc += 2
                    }
                }
                _ => unimplemented!(),
            },

            0xF => match ins.nn {
                0x07 => self.v[ins.x] = self.delay_timer,
                0x0A => {
                    // PAUSE until any keypressed then store in Vx
                    if !keys.iter().any(|&key| key) {
                        self.pc -= 2
                    } else {
                        for (i, key) in keys.iter().enumerate() {
                            if *key {
                                self.v[ins.x] = i as u8
                            }
                        }
                    }
                }
                0x15 => self.delay_timer = self.v[ins.x],
                0x18 => self.sound_timer = self.v[ins.x],
                0x1E => self.i += self.v[ins.x] as u16,
                0x29 => self.i = (0x50 + (self.v[ins.x] * 5)) as u16,
                0x33 => {
                    // store BCD in memory[i..i + 2]
                    self.memory[self.i as usize] = self.v[ins.x] / 100;
                    self.memory[self.i as usize + 1] = self.v[ins.x] % 100 / 10;
                    self.memory[self.i as usize + 2] = self.v[ins.x] % 10
                }
                0x55 => {
                    // store V0..Vx in memory[i..i + x]
                    for reg in 0..ins.x {
                        self.memory[self.i as usize + reg] = self.v[reg]
                    }
                }
                0x65 => {
                    // read V0..Vx from memory[i..i + x]
                    for reg in 0..ins.x {
                        self.v[reg] = self.memory[self.i as usize + reg]
                    }
                }
                _ => unimplemented!(),
            },

            _ => unimplemented!(),
        }
    }

    fn clear_pixels(&mut self) {
        for pixel in self.pixels.iter_mut() {
            *pixel = false
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
                let sprite_pixel = !matches!(sprite_row & (0x80 >> col), 0);

                let screen_pixel = &mut self.pixels
                    [((start_y + row) as u32 * SCREEN_WIDTH + (start_x + col) as u32) as usize];

                // if pixel is on in the sprite
                if sprite_pixel {
                    match (&sprite_pixel, &screen_pixel) {
                        (true, true) => {
                            *screen_pixel = false;
                            self.v[0xF] = 1
                        }
                        (true, false) => *screen_pixel = true,
                        (false, true) => *screen_pixel = true,
                        (false, false) => *screen_pixel = false,
                    }
                }
            }
        }
    }
}
