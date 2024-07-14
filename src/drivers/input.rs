use sdl2::{EventPump, Sdl};

pub struct InputDriver {
    event_pump: EventPump,
}

impl InputDriver {
    pub fn new(sdl_context: &Sdl) -> Self {
        Self {
            event_pump: sdl_context.event_pump().unwrap(),
        }
    }
}
