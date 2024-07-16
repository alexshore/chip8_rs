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
    pub fn new() -> Cpu {
        let mut res = Cpu {
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

    pub fn decrement_timers(&mut self) {
        self.sound_timer -= 1;
        self.delay_timer -= 1;
    }

    fn decode(&self) -> Instruction {
        Instruction::from(BigEndian::read_u16(&self.memory[self.pc..=self.pc + 1]))
    }

    fn execute(&mut self, ins: Instruction, keys: &[bool; 16]) {
        self.pc += 2;

        match ins.op {
            0x0 => match ins.nn {
                // Clear pixels
                0xE0 => {
                    self.clear_pixels();
                    self.update_display = true
                }

                // Return
                0xEE => self.pc = self.stack.pop().unwrap(),

                _ => unimplemented!(),
            },

            // PC = NNN
            0x1 => self.pc = ins.nnn as usize,

            // Call Subroutine NNN
            0x2 => {
                self.stack.push(self.pc);
                self.pc = ins.nnn as usize
            }

            // PC = PC + 2 IF Vx == nn
            0x3 if self.v[ins.x] == ins.nn => self.pc += 2,

            // PC = PC + 2 IF Vx != nn
            0x4 if self.v[ins.x] != ins.nn => self.pc += 2,

            // PC = PC + 2 IF Vx == Vy
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

                // Vx = Vx + Vy (overflow in VF)
                0x4 => {
                    let (res, overflow) = self.v[ins.x].overflowing_add(self.v[ins.y]);
                    self.v[0xF] = overflow as u8;
                    self.v[ins.x] = res;
                }

                // Vx = Vx - Vy (NOT overflow in VF)
                0x5 => {
                    let (res, overflow) = self.v[ins.x].overflowing_sub(self.v[ins.y]);
                    self.v[0xF] = !overflow as u8;
                    self.v[ins.x] = res;
                }

                // Vx = Vx >> 1 (overflow in VF)
                0x6 => {
                    self.v[0xF] = if self.v[ins.x] & 0b00000001 > 0 { 1 } else { 0 };
                    self.v[ins.x] >>= 1;
                }

                // Vx = Vy - Vx (NOT overflow in VF)
                0x7 => {
                    let (res, overflow) = self.v[ins.y].overflowing_sub(self.v[ins.x]);
                    self.v[0xF] = !overflow as u8;
                    self.v[ins.x] = res;
                }

                // Vx = Vx << 1 (overflow in VF)
                0xE => {
                    self.v[0xF] = if self.v[ins.x] & 0b10000000 > 0 { 1 } else { 0 };
                    self.v[ins.x] <<= 1;
                }

                _ => unimplemented!(),
            },

            // PC = PC + 2 IF Vx != Vy
            0x9 if ins.n == 0 => {
                if self.v[ins.x] != self.v[ins.y] {
                    self.pc += 2
                }
            }

            // I = nnn
            0xA => self.i = ins.nnn,

            // PC = nnn + V0
            0xB => self.pc = (ins.nnn + self.v[0] as u16) as usize,

            // Vx = rand(0-255) & nn
            0xC => self.v[ins.x] = self.rng.gen_range(0..255u8) & ins.nn,

            // Draw Sprite from I at (Vx, Vy)
            0xD => {
                self.draw_sprite(ins);
                self.update_display = true;
            }

            // keypress things :3
            0xE => match ins.nn {
                // PC = PC + 2 IF keys[Vx] == True
                0x9E => {
                    if keys[self.v[ins.x] as usize] {
                        self.pc += 2
                    }
                }
                // PC = PC + 2 IF keys[Vx] == False
                0xA1 => {
                    if !keys[self.v[ins.x] as usize] {
                        self.pc += 2
                    }
                }
                _ => unimplemented!(),
            },

            0xF => match ins.nn {
                // Vx = DT
                0x07 => self.v[ins.x] = self.delay_timer,

                // PAUSE until any keypressed then store in Vx
                0x0A => {
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

                // DT = Vx
                0x15 => self.delay_timer = self.v[ins.x],

                // ST = Vx
                0x18 => self.sound_timer = self.v[ins.x],

                // I = I + Vx
                0x1E => self.i += self.v[ins.x] as u16,

                // I = font[Vx] memory location
                0x29 => self.i = (0x50 + (self.v[ins.x] * 5)) as u16,

                // memory[i..i + 2] = Vx BCD
                0x33 => {
                    self.memory[self.i as usize] = self.v[ins.x] / 100;
                    self.memory[self.i as usize + 1] = self.v[ins.x] % 100 / 10;
                    self.memory[self.i as usize + 2] = self.v[ins.x] % 10
                }

                // memory[i..i + x] = V0..Vx
                0x55 => {
                    for reg in 0..ins.x {
                        self.memory[self.i as usize + reg] = self.v[reg]
                    }
                }

                // V0..Vx = memory[i..i + x]
                0x65 => {
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
            let sprite_byte = self.memory[self.i as usize + row as usize];

            for col in 0..8u8 {
                // match current bit in byte
                let sprite_bit = !matches!(sprite_byte & (0x80 >> col), 0);

                let screen_pixel = &mut self.pixels
                    [((start_y + row) as u32 * SCREEN_WIDTH + (start_x + col) as u32) as usize];

                // if pixel is on in the sprite
                if sprite_bit {
                    match (&sprite_bit, &screen_pixel) {
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
