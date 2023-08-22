use crate::{config::Config, game::GameClock, Model};
use pagurus::{
    event::{Event, KeyEvent},
    failure::OrFail,
    image::{Canvas, Color},
    spatial::{Position, Size},
};
use patican::{
    marker::Marker,
    spatial::{Point, RectangularArea},
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
    pub clock: GameClock,
}

impl ViewContext {
    fn scaled_window_size(&self) -> Size {
        self.window_size / self.scale() as u32
    }

    fn scale(&self) -> usize {
        // TODO: self.model.scale().get()
        1
    }

    fn to_position(&self, point: Point) -> Position {
        let center = self.scaled_window_size().to_region().center();
        let position = Position::from_xy(point.x as i32, point.y as i32) + center;

        // TODO
        // let position = (Position::from_xy(point.x as i32, point.y as i32) + center)
        //     - Position::from(self.model.camera().position);

        position * self.scale() as u32
    }

    fn visible_pixel_region(&self) -> RectangularArea {
        let center = self.scaled_window_size().to_region().center();
        let mut region = self.scaled_window_size().to_region();
        region.position = region.position - center; // TODO: + Position::from(self.model.camera().position);
        RectangularArea::from_points(
            [
                Point::new(region.start().x as i16, region.start().y as i16),
                Point::new(region.end().x as i16, region.end().y as i16),
            ]
            .into_iter(),
        )
    }
}

fn draw_pixel(ctx: &ViewContext, canvas: &mut Canvas, pixel_position: Point, color: patican::Rgba) {
    let color = Color::rgba(color.r, color.g, color.b, color.a);
    let p = ctx.to_position(pixel_position);
    for y in 0..ctx.scale() {
        for x in 0..ctx.scale() {
            canvas.draw_pixel(p.move_x(x as i32).move_y(y as i32), color);
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
        self.render_frames(ctx, canvas);
        self.canvas.render(ctx, canvas);
    }

    fn render_frames(&self, _ctx: &ViewContext, _canvas: &mut Canvas) {
        // TODO:
        // for frame in ctx.model.active_frames(ctx.clock) {
        //     for (pixel_position, color) in frame.pixels() {
        //         draw_pixel(ctx, canvas, pixel_position, color);
        //     }
        // }
    }

    fn render_background(&self, _ctx: &ViewContext, _canvas: &mut Canvas) {
        // TODO
        // match ctx.model.background() {
        //     Background::Color(c) => {
        //         canvas.fill_color(*c);
        //     }
        //     Background::Checkerboard(c) => {
        //         let n = c.dot_size.get() as i16;
        //         for pixel_position in ctx.visible_pixel_region().positions() {
        //             let color = if (pixel_position.x / n + pixel_position.y / n) % 2 == 0 {
        //                 c.color1
        //             } else {
        //                 c.color2
        //             };
        //             draw_pixel(ctx, canvas, pixel_position, color);
        //         }
        //     }
        // }
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
        // TODO
        // if ctx.model.has_stashed_pixels() {
        //     self.render_stashed_pixels(ctx, canvas);
        // }
        self.render_cursor(ctx, canvas);
    }

    fn render_pixels(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        let region = ctx.visible_pixel_region();

        for (pixel_position, color) in ctx.model.pixels().area(region) {
            draw_pixel(ctx, canvas, pixel_position, color);
        }
    }

    fn render_stashed_pixels(&self, _ctx: &ViewContext, _canvas: &mut Canvas) {
        // TODO
        // if ctx.now.as_millis() % 1000 < 500 {
        //     return;
        // }

        // for (pixel_position, color) in ctx.model.stashed_pixels() {
        //     draw_pixel(ctx, canvas, pixel_position, color);
        // }
    }

    fn render_marked_pixels(&self, ctx: &ViewContext, canvas: &mut Canvas, marker: &Marker) {
        let region = ctx.visible_pixel_region();
        for point in marker.marked_points() {
            if !region.contains(point) {
                continue;
            }

            // TODO: consider mark kind

            if ctx.now.as_millis() % 1000 < 500 {
                continue;
            }

            draw_pixel(ctx, canvas, point, ctx.model.brush_color().to_rgba());
        }
    }

    fn render_cursor(&self, ctx: &ViewContext, canvas: &mut Canvas) {
        // TODO: consider draw tool
        let mut c = ctx.model.brush_color().to_rgba();
        if !(ctx.now <= self.force_show_cursor_until || ctx.now.as_secs() % 2 == 0) {
            c = patican::Rgba::new(255 - c.r, 255 - c.g, 255 - c.b, c.a);
        };
        draw_pixel(ctx, canvas, ctx.model.cursor(), c);
    }

    fn handle_event(&mut self, ctx: &mut ViewContext, event: Event) -> pagurus::Result<()> {
        if let Event::Key(event) = event {
            self.handle_key_event(ctx, event).or_fail()?;
        }
        Ok(())
    }

    fn handle_key_event(&mut self, ctx: &mut ViewContext, key: KeyEvent) -> pagurus::Result<()> {
        match ctx.config.key.get_command(key) {
            None => {}
            Some(commands) => {
                for command in commands {
                    // TODO
                    // if matches!(command, Command::Quit) {
                    //     ctx.quit = true;
                    // }
                    ctx.model.apply(command).or_fail()?;
                }
                self.force_show_cursor_until = ctx.now + Duration::from_millis(500);
            }
        }
        Ok(())
    }
}
