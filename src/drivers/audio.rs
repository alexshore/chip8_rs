pub struct AudioDriver {}

impl AudioDriver {
    pub fn new() -> Self {
        Self {}
    }

    pub fn beep(&self) {
        println!("beep")
    }
}
