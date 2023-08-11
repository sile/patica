use crate::{
    config::{Config, KeyCommand},
    model::Model,
};
use pagurus::{
    event::{Event, KeyEvent},
    failure::OrFail,
    image::{Canvas, Color},
    spatial::{Position, Size},
};
use std::sync::Arc;
use std::time::Duration;

const COLOR_BG0: Color = Color::rgb(100, 100, 100);
const COLOR_BG1: Color = Color::rgb(200, 200, 200);

#[derive(Debug)]
pub struct ViewContext {
    pub window_size: Size,
    pub now: Duration,
    pub model: Model,
    pub config: Arc<Config>,
    pub quit: bool,
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
    force_show_cursor_until: Duration,
}

impl PixelCanvas {
    fn render(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        self.render_pixels(ctx, canvas);
        self.render_cursor(ctx, canvas);
    }

    fn render_pixels(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        let center = ctx.window_size.to_region().center();
        for (pixel_position, color) in ctx.model.visible_pixels(ctx.window_size) {
            let position = Position::from(pixel_position) + center;
            canvas.draw_pixel(position, color);
        }
    }

    fn render_cursor(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        let mut color = ctx.model.active_color();
        if !(ctx.now <= self.force_show_cursor_until || ctx.now.as_secs() % 2 == 0) {
            let c = color.to_rgba();
            color = Color::rgba(255 - c.r, 255 - c.g, 255 - c.b, c.a);
        };
        canvas.draw_pixel(self.cursor_position(ctx), color);
    }

    fn cursor_position(&self, ctx: &ViewContext) -> Position {
        // TODO: consider camera position
        let mut position = ctx.window_size.to_region().center();
        position.x += ctx.model.cursor().x() as i32;
        position.y += ctx.model.cursor().y() as i32;
        position
    }

    fn handle_event(&mut self, ctx: &mut ViewContext, event: Event) -> pagurus::Result<()> {
        match event {
            Event::Key(event) => self.handle_key_event(ctx, event).or_fail()?,
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, ctx: &mut ViewContext, key: KeyEvent) -> pagurus::Result<()> {
        match ctx.config.key.get_command(key) {
            None => {}
            Some(KeyCommand::Quit) => {
                ctx.quit = true;
            }
            Some(KeyCommand::Model(command)) => {
                ctx.model.apply(command).or_fail()?;
                self.force_show_cursor_until = ctx.now + Duration::from_millis(500);
            }
        }
        Ok(())
    }
}
