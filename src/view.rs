use crate::{config::KeyConfig, model::Model};
use orfail::OrFail;
use pagurus::{
    event::Event,
    image::Canvas,
    spatial::{Position, Size},
};
use pati::{Color, Point};
use std::{collections::BTreeSet, time::Duration};

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

        let marked_points = self.collect_marked_points(model);

        self.render_pixels(model, canvas, &marked_points);
        self.cursor.render(model, canvas, &marked_points);
    }

    fn collect_marked_points(&self, model: &Model) -> BTreeSet<Point> {
        let mut points = BTreeSet::new();
        if let Some(marker) = model.marker() {
            points.extend(marker.marked_points());
        } else {
            points.insert(model.cursor());
        }
        points
    }

    fn render_pixels(
        &self,
        model: &Model,
        canvas: &mut WindowCanvas,
        marked_points: &BTreeSet<Point>,
    ) {
        let top_left = canvas.position_to_point(model, Position::ORIGIN);
        let bottom_right = canvas.position_to_point(model, canvas.window_size.to_region().end());
        for (point, color) in model.canvas().range_pixels(top_left..bottom_right) {
            if marked_points.contains(&point) {
                continue;
            }
            canvas.dot(model, point, color);
        }
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

// TODO: rename
#[derive(Debug, Default, Clone, Copy)]
struct Cursor {
    show: bool,
    switch_time: Duration,
}

impl Cursor {
    fn render(self, model: &Model, canvas: &mut WindowCanvas, marked_points: &BTreeSet<Point>) {
        let color = model.brush_color();
        if self.show {
            if let Some(editor) = model.editor() {
                for (point, color) in editor.pixels() {
                    canvas.dot(model, point + model.cursor(), color);
                }
            } else {
                for &point in marked_points {
                    canvas.dot(model, point, color);
                }
            }
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
        let scale = model.scale().get() as u32;
        let color = pagurus::image::Color::rgba(color.r, color.g, color.b, color.a);
        let p = self.point_to_position(model, point);
        for y in 0..scale {
            for x in 0..scale {
                self.canvas
                    .draw_pixel(p.move_x(x as i32).move_y(y as i32), color);
            }
        }
    }

    fn point_to_position(&self, model: &Model, point: Point) -> Position {
        let scale = model.scale().get() as u32;
        let center = (self.window_size / scale).to_region().center();
        let point_position = Position::from_xy(point.x as i32, point.y as i32);
        let camera_position = Position::from_xy(model.camera().x as i32, model.camera().y as i32);
        (point_position - camera_position + center) * scale
    }

    fn position_to_point(&self, model: &Model, position: Position) -> Point {
        let scale = model.scale().get() as u32;
        let center = (self.window_size / scale).to_region().center();
        let camera_position = Position::from_xy(model.camera().x as i32, model.camera().y as i32);
        let p = (position / scale) + camera_position - center;
        Point::new(p.x as i16, p.y as i16)
    }
}
