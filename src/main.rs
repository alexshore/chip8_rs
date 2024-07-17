#![allow(unused_imports)]

mod cpu;
mod drivers;

use std::path::PathBuf;

use clap::Parser;
use cpu::Cpu;
use drivers::{check_timers, AudioDriver, DisplayDriver, InputDriver, Timer};

const SCREEN_WIDTH: u32 = 64;
const SCREEN_HEIGHT: u32 = 32;
const PIXEL_SIZE: u32 = 10;

#[derive(PartialEq, Eq)]
enum State {
    Play,
    Pause,
    Exit,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Execute {
    Cpu,
    Display,
    Sound,
    Delay,
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

    let mut timers: Vec<Timer> = vec![
        Timer::new(1000, Execute::Cpu),
        Timer::new(60, Execute::Display),
        Timer::new(60, Execute::Sound),
        Timer::new(60, Execute::Delay),
    ];

    'mainloop: loop {
        if input_driver.get_inputs(&mut keys) == State::Exit {
            break 'mainloop;
        }

        let execute = check_timers(&mut timers);

        if execute.contains(&Execute::Cpu) {
            cpu.tick(&keys);
        }

        if execute.contains(&Execute::Display) {
            display_driver.draw(&cpu.pixels)
        }
    }
}
