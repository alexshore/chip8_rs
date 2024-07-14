use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;

use crate::{PIXEL_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct DisplayDriver {
    canvas: WindowCanvas,
}

impl DisplayDriver {
    pub fn new(sdl_context: &Sdl) -> Self {
        let subsystem = sdl_context.video().unwrap();
        let window = subsystem
            .window(
                "CHIP-8",
                SCREEN_WIDTH * PIXEL_SIZE,
                SCREEN_HEIGHT * PIXEL_SIZE,
            )
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Self { canvas }
    }
}
