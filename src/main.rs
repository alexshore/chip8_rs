#![allow(unused_imports)]

mod cpu;
mod drivers;

use std::{path::PathBuf, time::Instant};

use clap::Parser;
use cpu::{Cpu, State};
use drivers::{check_timers, AudioDriver, DisplayDriver, InputDriver, Timer};

const SCREEN_WIDTH: u32 = 64;
const SCREEN_HEIGHT: u32 = 32;
const PIXEL_SIZE: u32 = 10;

#[derive(PartialEq, Eq)]
enum Event {
    Toggle,
    Reset,
    Exit,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum DoTick {
    Cpu,
    Display,
    SoundDelay,
}

#[derive(Parser)]
#[command(name = "CHIP8_RS", version = "1.0", about)]
struct Args {
    /// ROM file
    filename: PathBuf,

    /// CPU frequency (default: 1000Hz)
    #[arg(short = 'f', long)]
    freq: Option<u32>,
}

fn main() {
    let args = Args::parse();

    let sdl_context = sdl2::init().unwrap();

    let audio_driver = AudioDriver::new();
    let mut display_driver = DisplayDriver::new(&sdl_context);
    let mut input_driver = InputDriver::new(&sdl_context);

    let mut timers: Vec<Timer> = vec![
        Timer::new(args.freq.unwrap_or(1000), DoTick::Cpu),
        Timer::new(60, DoTick::Display),
        Timer::new(60, DoTick::SoundDelay),
    ];

    let mut cpu = Cpu::new(&args.filename);

    'mainloop: loop {
        match input_driver.get_inputs(&mut cpu.keys) {
            Some(Event::Toggle) => cpu.toggle_state(),
            Some(Event::Reset) => cpu = Cpu::new(&args.filename),
            Some(Event::Exit) => break 'mainloop,
            None => (),
        }

        let execute = check_timers(&mut timers);

        if !cpu.is_paused() && execute.contains(&DoTick::Cpu) {
            cpu.tick();
        }

        if execute.contains(&DoTick::Display) {
            display_driver.draw(&cpu.pixels)
        }

        if !cpu.is_paused() && execute.contains(&DoTick::SoundDelay) {
            cpu.decrement_timers()
        }

        if cpu.sound_timer > 0 {
            audio_driver.beep()
        }
    }
}
