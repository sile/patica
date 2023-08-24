use crate::{config::KeyConfig, model::Model};
use orfail::OrFail;
use pagurus::{
    event::Event,
    image::Canvas,
    spatial::{Position, Size},
};
use pati::{Color, Point};
use std::time::Duration;

// #[derive(Debug)]
// pub struct ViewContext {
// pub now: Duration,
// pub quit: bool,
// pub clock: GameClock,
//}

// impl ViewContext {
//     fn scaled_window_size(&self) -> Size {
//         self.window_size / self.scale() as u32
//     }

//     fn scale(&self) -> usize {
//         // TODO: self.model.scale().get()
//         1
//     }

//     fn to_position(&self, point: Point) -> Position {
//         let center = self.scaled_window_size().to_region().center();
//         let position = Position::from_xy(point.x as i32, point.y as i32) + center;

//         // TODO
//         // let position = (Position::from_xy(point.x as i32, point.y as i32) + center)
//         //     - Position::from(self.model.camera().position);

//         position * self.scale() as u32
//     }

//     fn visible_pixel_region(&self) -> RectangularArea {
//         let center = self.scaled_window_size().to_region().center();
//         let mut region = self.scaled_window_size().to_region();
//         region.position = region.position - center; // TODO: + Position::from(self.model.camera().position);
//         RectangularArea::from_points(
//             [
//                 Point::new(region.start().x as i16, region.start().y as i16),
//                 Point::new(region.end().x as i16, region.end().y as i16),
//             ]
//             .into_iter(),
//         )
//     }
// }

#[derive(Debug, Default)]
pub struct View {
    key_config: KeyConfig,
    cursor: Cursor,
}

impl View {
    pub fn set_key_config(&mut self, config: KeyConfig) {
        self.key_config = config;
    }

    pub fn render(&self, model: &Model, canvas: &mut WindowCanvas) {
        self.render_background(canvas);
        self.cursor.render(model, canvas);
        // self.render_frames(ctx, canvas);
        // self.canvas.render(ctx, canvas);
    }

    fn render_background(&self, canvas: &mut WindowCanvas) {
        // TODO
        canvas.canvas.fill_color(pagurus::image::Color::WHITE);
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

    pub fn handle_event(&mut self, model: &mut Model, event: Event) -> orfail::Result<()> {
        self.cursor.handle_event(model, event).or_fail()?;

        let Event::Key(key) = event else {
            return Ok(());
        };
        for command in self.key_config.get_commands(key) {
            model.apply(command);
            // TODO: self.force_show_cursor_until = ctx.now + Duration::from_millis(500);
        }

        Ok(())
    }
}

// impl View {

//     fn render_frames(&self, _ctx: &ViewContext, _canvas: &mut Canvas) {
//         // TODO:
//         // for frame in ctx.model.active_frames(ctx.clock) {
//         //     for (pixel_position, color) in frame.pixels() {
//         //         draw_pixel(ctx, canvas, pixel_position, color);
//         //     }
//         // }
//     }

// }

// #[derive(Debug, Default)]
// pub struct PixelCanvas {
//     force_show_cursor_until: Duration,
// }

// impl PixelCanvas {
//     fn render(&self, ctx: &ViewContext, canvas: &mut Canvas) {
//         self.render_pixels(ctx, canvas);
//         if let Some(marker) = ctx.model.marker() {
//             self.render_marked_pixels(ctx, canvas, marker);
//         }
//         // TODO
//         // if ctx.model.has_stashed_pixels() {
//         //     self.render_stashed_pixels(ctx, canvas);
//         // }
//         self.render_cursor(ctx, canvas);
//     }

//     fn render_pixels(&self, ctx: &ViewContext, canvas: &mut Canvas) {
//         let region = ctx.visible_pixel_region();

//         for (pixel_position, color) in ctx.model.pixels().area(region) {
//             draw_pixel(ctx, canvas, pixel_position, color);
//         }
//     }

//     fn render_stashed_pixels(&self, _ctx: &ViewContext, _canvas: &mut Canvas) {
//         // TODO
//         // if ctx.now.as_millis() % 1000 < 500 {
//         //     return;
//         // }

//         // for (pixel_position, color) in ctx.model.stashed_pixels() {
//         //     draw_pixel(ctx, canvas, pixel_position, color);
//         // }
//     }

//     fn render_marked_pixels(&self, ctx: &ViewContext, canvas: &mut Canvas, marker: &Marker) {
//         let region = ctx.visible_pixel_region();
//         for point in marker.marked_points() {
//             if !region.contains(point) {
//                 continue;
//             }

//             // TODO: consider mark kind

//             if ctx.now.as_millis() % 1000 < 500 {
//                 continue;
//             }

//             draw_pixel(ctx, canvas, point, ctx.model.brush_color().to_rgba());
//         }
//     }

//     fn render_cursor(&self, ctx: &ViewContext, canvas: &mut Canvas) {
//         // TODO: consider draw tool
//         let mut c = ctx.model.brush_color().to_rgba();
//         if !(ctx.now <= self.force_show_cursor_until || ctx.now.as_secs() % 2 == 0) {
//             c = pati::Rgba::new(255 - c.r, 255 - c.g, 255 - c.b, c.a);
//         };
//         draw_pixel(ctx, canvas, ctx.model.cursor(), c);
//     }
// }

// fn dot(ctx: &ViewContext, canvas: &mut Canvas, point: Point, color: Color) {
//     let color = pagurus::image::Color::rgba(color.r, color.g, color.b, color.a);
//     let p = ctx.to_window_position(point);
//     for y in 0..ctx.scale() {
//         for x in 0..ctx.scale() {
//             canvas.draw_pixel(p.move_x(x as i32).move_y(y as i32), color);
//         }
//     }
// }

fn to_position(point: Point) -> Position {
    Position::from_xy(point.x as i32, point.y as i32)
}

#[derive(Debug, Default, Clone, Copy)]
struct Cursor {
    show: bool,
    switch_time: Duration,
}

impl Cursor {
    fn render(self, model: &Model, canvas: &mut WindowCanvas) {
        let cursor = model.cursor();
        let color = model.brush_color();
        if self.show {
            canvas.dot(model, cursor, color);
        }
    }

    fn handle_event(&mut self, model: &mut Model, event: Event) -> orfail::Result<()> {
        if matches!(event, Event::Key(_)) {
            self.show = true;
            self.switch_time = model.clock().duration() + Duration::from_secs(1);
        }

        if model.clock().duration() >= self.switch_time {
            self.show = !self.show;
            self.switch_time = model.clock().duration() + Duration::from_millis(500);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct WindowCanvas<'a> {
    canvas: Canvas<'a>,
    window_size: Size,
}

impl<'a> WindowCanvas<'a> {
    pub fn new(canvas: Canvas<'a>, window_size: Size) -> Self {
        Self {
            canvas,
            window_size,
        }
    }

    fn dot(&mut self, model: &Model, point: Point, color: Color) {
        let color = pagurus::image::Color::rgba(color.r, color.g, color.b, color.a);
        let p = self.to_window_position(model, point);
        for y in 0..model.scale().get() {
            for x in 0..model.scale().get() {
                self.canvas
                    .draw_pixel(p.move_x(x as i32).move_y(y as i32), color);
            }
        }
    }

    fn to_window_position(&self, model: &Model, point: Point) -> Position {
        let center = self.window_size.to_region().center();
        to_position(point) + center - to_position(model.camera())
    }
}
