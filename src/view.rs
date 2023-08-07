use crate::model::PixelPosition;
use pagurus::{
    image::{Canvas, Color},
    spatial::{Position, Size},
};
use std::time::Duration;

const COLOR_BG0: Color = Color::rgb(100, 100, 100);
const COLOR_BG1: Color = Color::rgb(200, 200, 200);

// TODO
const COLOR_CURSOR: Color = Color::rgb(255, 0, 0);

#[derive(Debug)]
pub struct ViewContext {
    pub window_size: Size,
    pub now: Duration,
}

impl ViewContext {
    pub fn new(window_size: Size, now: Duration) -> Self {
        Self { window_size, now }
    }
}

#[derive(Debug, Default)]
pub struct View {
    canvas: PixelCanvas,
}

impl View {
    pub fn render(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        self.render_background(canvas);
        self.canvas.render(ctx, canvas);
    }

    fn render_background(&self, canvas: &mut Canvas) {
        for position in canvas.drawing_region().iter() {
            let color = if (position.x + position.y) % 2 == 0 {
                COLOR_BG0
            } else {
                COLOR_BG1
            };
            canvas.draw_pixel(position, color);
        }
    }
}

#[derive(Debug, Default)]
pub struct PixelCanvas {
    cursor: PixelPosition,
    camera: PixelPosition,
}

impl PixelCanvas {
    fn render(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        if ctx.now.as_secs() % 2 == 0 {
            canvas.draw_pixel(self.cursor_position(ctx), COLOR_CURSOR)
        }
    }

    fn cursor_position(&self, ctx: &ViewContext) -> Position {
        let mut position = ctx.window_size.to_region().center();
        position.x += self.cursor.x as i32;
        position.y += self.cursor.y as i32;
        position
    }
}
