use sdl2::pixels::Color;
use sdl2::rect::Rect;
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

    pub fn draw(&mut self, pixels: &[bool]) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));

        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                if pixels[(y * SCREEN_WIDTH + x) as usize] {
                    let _ = self.canvas.fill_rect(Rect::new(
                        (x * PIXEL_SIZE) as i32,
                        (y * PIXEL_SIZE) as i32,
                        PIXEL_SIZE,
                        PIXEL_SIZE,
                    ));
                }
            }
        }
        self.canvas.present()
    }
}
