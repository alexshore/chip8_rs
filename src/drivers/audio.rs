use sdl2::Sdl;

pub struct AudioDriver {}

impl AudioDriver {
    pub fn new(sdl_context: &Sdl) -> Self {
        Self {}
    }

    pub fn beep(&self) {
        println!("BEEP!!!")
    }
}
