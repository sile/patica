use crate::{
    config::Config,
    model::{Background, Command, Marker, Model, PixelPosition, PixelRegion, PixelSize},
};
use pagurus::{
    event::{Event, KeyEvent},
    failure::OrFail,
    image::{Canvas, Color},
    spatial::{Position, Size},
};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
pub struct ViewContext {
    pub window_size: Size,
    pub now: Duration,
    pub model: Model,
    pub config: Arc<Config>,
    pub quit: bool,
}

impl ViewContext {
    fn to_canvas_position(&self, pixel_position: PixelPosition) -> Position {
        let center = self.window_size.to_region().center();
        (Position::from(pixel_position) + center) - Position::from(self.model.camera().position)
    }

    fn visible_pixel_region(&self) -> PixelRegion {
        let center = self.window_size.to_region().center();
        let mut region = self.window_size.to_region();
        region.position = (region.position - center) + Position::from(self.model.camera().position);
        PixelRegion {
            position: PixelPosition {
                x: region.position.x as i16,
                y: region.position.y as i16,
            },
            size: PixelSize {
                width: region.size.width as u16,
                height: region.size.height as u16,
            },
        }
    }
}

#[derive(Debug, Default)]
pub struct View {
    canvas: PixelCanvas,
}

impl View {
    pub fn render(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        self.render_background(ctx, canvas);
        self.canvas.render(ctx, canvas);
    }

    fn render_background(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        match ctx.model.background() {
            Background::Color(c) => {
                canvas.fill_color(*c);
            }
            Background::Checkerboard(c) => {
                let n = c.dot_size.get() as i32;
                for position in canvas.drawing_region().iter() {
                    let color = if (position.x / n + position.y / n) % 2 == 0 {
                        c.color1
                    } else {
                        c.color2
                    };
                    canvas.draw_pixel(position, color);
                }
            }
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
        if let Some(marker) = ctx.model.marker() {
            self.render_marked_pixels(ctx, canvas, marker);
        }
        if ctx.model.has_stashed_pixels() {
            self.render_stashed_pixels(ctx, canvas);
        }
        self.render_cursor(ctx, canvas);
    }

    fn render_pixels(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        let region = ctx.visible_pixel_region();

        // TODO: optimzie
        for pixel_position in region.positions() {
            if let Some(color) = ctx.model.get_pixel_color(pixel_position) {
                canvas.draw_pixel(ctx.to_canvas_position(pixel_position), color);
            }
        }
    }

    fn render_stashed_pixels(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        if ctx.now.as_millis() % 1000 < 500 {
            return;
        }

        for (pixel_position, color) in ctx.model.stashed_pixels() {
            canvas.draw_pixel(ctx.to_canvas_position(pixel_position), color);
        }
    }

    fn render_marked_pixels(&self, ctx: &ViewContext, canvas: &mut Canvas, marker: &Marker) {
        let region = ctx.visible_pixel_region();
        for pixel_position in marker.marked_pixels() {
            if !region.contains(pixel_position) {
                continue;
            }

            // TODO: consider mark kind

            if ctx.now.as_millis() % 1000 < 500 {
                continue;
            }

            canvas.draw_pixel(
                ctx.to_canvas_position(pixel_position),
                ctx.model.dot_color(),
            );
        }
    }

    fn render_cursor(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        // TODO: consider draw tool
        let mut color = ctx.model.dot_color();
        if !(ctx.now <= self.force_show_cursor_until || ctx.now.as_secs() % 2 == 0) {
            let c = color.to_rgba();
            color = Color::rgba(255 - c.r, 255 - c.g, 255 - c.b, c.a);
        };
        canvas.draw_pixel(self.cursor_position(ctx), color);
    }

    fn cursor_position(&self, ctx: &ViewContext) -> Position {
        ctx.to_canvas_position(ctx.model.cursor().position())
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
            Some(commands) => {
                for command in commands.into_iter() {
                    if matches!(command, Command::Quit) {
                        ctx.quit = true;
                    }
                    ctx.model.apply(command).or_fail()?;
                }
                self.force_show_cursor_until = ctx.now + Duration::from_millis(500);
            }
        }
        Ok(())
    }
}
