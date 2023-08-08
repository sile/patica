use crate::model::PixelPosition;
use pagurus::{
    event::{Event, Key, KeyEvent, MouseEvent},
    failure::OrFail,
    image::{Canvas, Color},
    spatial::{Position, Size},
};
use std::time::Duration;

const COLOR_BG0: Color = Color::rgb(100, 100, 100);
const COLOR_BG1: Color = Color::rgb(200, 200, 200);

// TODO
const COLOR_CURSOR: Color = Color::rgba(255, 0, 0, 100);

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

    pub fn handle_event(&mut self, ctx: &ViewContext, event: Event) -> pagurus::Result<()> {
        self.canvas.handle_event(ctx, event).or_fail()?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct PixelCanvas {
    cursor: PixelPosition,
    camera: PixelPosition,
    force_show_cursor_until: Duration,
}

impl PixelCanvas {
    fn render(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        if ctx.now <= self.force_show_cursor_until || ctx.now.as_secs() % 2 == 0 {
            canvas.draw_pixel(self.cursor_position(ctx), COLOR_CURSOR)
        }
    }

    fn cursor_position(&self, ctx: &ViewContext) -> Position {
        let mut position = ctx.window_size.to_region().center();
        position.x += self.cursor.x as i32;
        position.y += self.cursor.y as i32;
        position
    }

    fn handle_event(&mut self, ctx: &ViewContext, event: Event) -> pagurus::Result<()> {
        match event {
            Event::Key(event) => self.handle_key_event(ctx, event).or_fail()?,
            Event::Mouse(event) => self.handle_mouse_event(ctx, event).or_fail()?,
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        ctx: &ViewContext,
        KeyEvent { key, .. }: KeyEvent,
    ) -> pagurus::Result<()> {
        // TODO: max / min
        match key {
            Key::Up => {
                self.cursor.y -= 1;
                self.force_show_cursor_until = ctx.now + Duration::from_millis(500);
            }
            Key::Down => {
                self.cursor.y += 1;
                self.force_show_cursor_until = ctx.now + Duration::from_millis(500);
            }
            Key::Left => {
                self.cursor.x -= 1;
                self.force_show_cursor_until = ctx.now + Duration::from_millis(500);
            }
            Key::Right => {
                self.cursor.x += 1;
                self.force_show_cursor_until = ctx.now + Duration::from_millis(500);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_mouse_event(&mut self, ctx: &ViewContext, event: MouseEvent) -> pagurus::Result<()> {
        // TODO:
        self.cursor = PixelPosition {
            x: event.position().x as i16,
            y: event.position().y as i16,
        };
        self.force_show_cursor_until = ctx.now + Duration::from_millis(500);
        Ok(())
    }
}
