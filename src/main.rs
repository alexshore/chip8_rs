mod cpu;
mod drivers;

use std::path::PathBuf;

use clap::Parser;
use cpu::Cpu;
use drivers::{AudioDriver, DisplayDriver, InputDriver};

const SCREEN_WIDTH: u32 = 64;
const SCREEN_HEIGHT: u32 = 32;
const PIXEL_SIZE: u32 = 10;

#[derive(PartialEq, Eq)]
enum EmuState {
    Play,
    Pause,
    Exit,
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// ROM file
    filename: PathBuf,
}

fn main() {
    let args = Args::parse();

    let sdl_context = sdl2::init().unwrap();

    let mut display_driver = DisplayDriver::new(&sdl_context);
    let mut audio_driver = AudioDriver::new(&sdl_context);
    let mut input_driver = InputDriver::new(&sdl_context);

    let mut cpu = Cpu::new();
    cpu.load_rom(&args.filename);

    let mut keys = [false; 16];

    'mainloop: loop {
        if input_driver.get_inputs(&mut keys) == EmuState::Exit {
            break 'mainloop;
        }

        if keys.iter().any(|&key| key) {
            println!("{:?}", keys);
        }

        cpu.tick(&keys);

        if cpu.update_display {
            display_driver.draw(&cpu.pixels)
        }
    }
}
