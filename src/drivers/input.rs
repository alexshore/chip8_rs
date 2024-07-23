use sdl2::event::Event::KeyDown;
use sdl2::keyboard::{KeyboardState, Keycode, Scancode};
use sdl2::EventPump;
use sdl2::Sdl;

use crate::Event;

const SCANCODES: [Scancode; 16] = [
    Scancode::Num1,
    Scancode::Num2,
    Scancode::Num3,
    Scancode::Num4,
    Scancode::Q,
    Scancode::W,
    Scancode::E,
    Scancode::R,
    Scancode::A,
    Scancode::S,
    Scancode::D,
    Scancode::F,
    Scancode::Z,
    Scancode::X,
    Scancode::C,
    Scancode::V,
];

pub struct InputDriver {
    event_pump: EventPump,
}

impl InputDriver {
    pub fn new(sdl_context: &Sdl) -> Self {
        Self {
            event_pump: sdl_context.event_pump().unwrap(),
        }
    }

    pub fn get_inputs(&mut self, keys: &mut [bool; 16]) -> Option<Event> {
        let keyboardstate = KeyboardState::new(&self.event_pump);
        for (i, key) in SCANCODES.iter().enumerate() {
            keys[i] = keyboardstate.is_scancode_pressed(*key)
        }

        for event in self.event_pump.poll_iter() {
            match event {
                KeyDown {
                    keycode: Some(Keycode::SPACE),
                    ..
                } => return Some(Event::Toggle),
                KeyDown {
                    keycode: Some(Keycode::ESCAPE),
                    ..
                } => return Some(Event::Exit),
                KeyDown {
                    keycode: Some(Keycode::BACKSPACE),
                    ..
                } => return Some(Event::Reset),
                _ => (),
            }
        }
        None
    }
}
