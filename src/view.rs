use crate::model::{Model, PixelPosition};
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
    pub model: Model,
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

    pub fn handle_event(&mut self, ctx: &mut ViewContext, event: Event) -> pagurus::Result<()> {
        self.canvas.handle_event(ctx, event).or_fail()?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct PixelCanvas {
    camera: PixelPosition,
    force_show_cursor_until: Duration,
}

impl PixelCanvas {
    fn render(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        if ctx.now <= self.force_show_cursor_until || ctx.now.as_secs() % 2 == 0 {
            canvas.draw_pixel(self.cursor_position(ctx), COLOR_CURSOR);
        }
    }

    fn cursor_position(&self, ctx: &ViewContext) -> Position {
        let mut position = ctx.window_size.to_region().center();
        position.x += ctx.model.cursor().x() as i32;
        position.y += ctx.model.cursor().y() as i32;
        position
    }

    fn handle_event(&mut self, ctx: &mut ViewContext, event: Event) -> pagurus::Result<()> {
        match event {
            Event::Key(event) => self.handle_key_event(ctx, event).or_fail()?,
            Event::Mouse(event) => self.handle_mouse_event(ctx, event).or_fail()?,
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(
        &mut self,
        ctx: &mut ViewContext,
        KeyEvent { key, .. }: KeyEvent,
    ) -> pagurus::Result<()> {
        match key {
            Key::Up => {
                self.move_cursor(ctx, (0, -1).into()).or_fail()?;
            }
            Key::Down => {
                self.move_cursor(ctx, (0, 1).into()).or_fail()?;
            }
            Key::Left => {
                self.move_cursor(ctx, (-1, 0).into()).or_fail()?;
            }
            Key::Right => {
                self.move_cursor(ctx, (1, 0).into()).or_fail()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn move_cursor(&mut self, ctx: &mut ViewContext, delta: PixelPosition) -> pagurus::Result<()> {
        // TODO: max / min

        let command = ctx.model.move_cursor_command(delta.into());
        ctx.model.apply(command).or_fail()?;
        self.force_show_cursor_until = ctx.now + Duration::from_millis(500);
        Ok(())
    }

    fn handle_mouse_event(
        &mut self,
        _ctx: &mut ViewContext,
        _event: MouseEvent,
    ) -> pagurus::Result<()> {
        Ok(())
    }
}
