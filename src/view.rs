use crate::{model::Model, screen::Screen};
use pagurus::{event::Event, System};

#[derive(Debug, Default)]
pub struct View {}

impl View {
    pub fn render(&self, model: &Model, screen: &mut Screen) {}

    pub fn handle_event<S: System>(
        &mut self,
        system: &S,
        model: &mut Model,
        event: Event,
    ) -> orfail::Result<()> {
        //         self.cursor.handle_event(system, model, event).or_fail()?;

        //         let Event::Key(key) = event else {
        //             return Ok(());
        //         };
        //         for command in self.key_config.get_commands(key) {
        //             model.apply(command);
        //         }

        Ok(())
    }
}

// use crate::{config::KeyConfig, model::Model};
// use orfail::OrFail;
// use pagurus::{
//     event::Event,
//     image::Canvas,
//     spatial::{Position, Size},
//     System,
// };
// use pati::{Color, Point};
// use std::{collections::BTreeSet, time::Duration};

// #[derive(Debug, Default)]
// pub struct View {
//     key_config: KeyConfig,
//     cursor: Cursor,
// }

// impl View {
//     pub fn set_key_config(&mut self, config: KeyConfig) {
//         self.key_config = config;
//     }

//     pub fn render(&self, model: &Model, canvas: &mut WindowCanvas) {
//         self.render_background(model, canvas);
//         self.render_frames(model, canvas);

//         let marked_points = self.collect_marked_points(model);
//         self.render_pixels(model, canvas, &marked_points);
//         self.cursor.render(model, canvas, &marked_points);
//     }

//     fn render_frames(&self, model: &Model, canvas: &mut WindowCanvas) {
//         for frame in model
//             .frames()
//             .values()
//             .filter(|f| f.frame.is_visible(model.ticks()))
//         {
//             for (&point, &color) in frame.pixels.iter() {
//                 canvas.dot(model, point, color);
//             }
//         }
//     }

//     fn collect_marked_points(&self, model: &Model) -> BTreeSet<Point> {
//         let mut points = BTreeSet::new();
//         if let Some(marker) = model.marker() {
//             points.extend(marker.marked_points());
//         } else {
//             points.insert(model.cursor());
//         }
//         points
//     }

//     fn render_pixels(
//         &self,
//         model: &Model,
//         canvas: &mut WindowCanvas,
//         marked_points: &BTreeSet<Point>,
//     ) {
//         let top_left = canvas.position_to_point(model, Position::ORIGIN);
//         let bottom_right = canvas.position_to_point(model, canvas.window_size.to_region().end());
//         for (point, color) in model.canvas().range_pixels(top_left..bottom_right) {
//             if marked_points.contains(&point) {
//                 continue;
//             }
//             canvas.dot(model, point, color);
//         }
//     }

//     fn render_background(&self, model: &Model, canvas: &mut WindowCanvas) {
//         let c = model.background_color();
//         canvas
//             .canvas
//             .fill_color(pagurus::image::Color::rgba(c.r, c.g, c.b, c.a));
//     }

// }

// // TODO: rename
// #[derive(Debug, Default, Clone, Copy)]
// struct Cursor {
//     show: bool,
//     switch_time: Duration,
// }

// impl Cursor {
//     fn render(self, model: &Model, canvas: &mut WindowCanvas, marked_points: &BTreeSet<Point>) {
//         let color = model.brush_color();
//         if self.show {
//             if let Some(editor) = model.editor() {
//                 for (point, color) in editor.pixels() {
//                     canvas.dot(model, point + model.cursor(), color);
//                 }
//             } else {
//                 for &point in marked_points {
//                     canvas.dot(model, point, color);
//                 }
//             }
//         }
//     }

//     fn handle_event<S: System>(
//         &mut self,
//         system: &S,
//         _model: &mut Model,
//         event: Event,
//     ) -> orfail::Result<()> {
//         if matches!(event, Event::Key(_)) {
//             self.show = true;
//             self.switch_time = system.clock_game_time() + Duration::from_secs(1);
//         }

//         if system.clock_game_time() >= self.switch_time {
//             self.show = !self.show;
//             self.switch_time = system.clock_game_time() + Duration::from_millis(500);
//         }

//         Ok(())
//     }
// }

// #[derive(Debug)]
// pub struct WindowCanvas<'a> {
//     canvas: Canvas<'a>,
//     window_size: Size,
// }

// impl<'a> WindowCanvas<'a> {
//     pub fn new(canvas: Canvas<'a>, window_size: Size) -> Self {
//         Self {
//             canvas,
//             window_size,
//         }
//     }

//     fn dot(&mut self, model: &Model, point: Point, color: Color) {
//         let scale = model.scale().get() as u32;
//         let color = pagurus::image::Color::rgba(color.r, color.g, color.b, color.a);
//         let p = self.point_to_position(model, point);
//         for y in 0..scale {
//             for x in 0..scale {
//                 self.canvas
//                     .draw_pixel(p.move_x(x as i32).move_y(y as i32), color);
//             }
//         }
//     }

//     fn point_to_position(&self, model: &Model, point: Point) -> Position {
//         let scale = model.scale().get() as u32;
//         let center = (self.window_size / scale).to_region().center();
//         let point_position = Position::from_xy(point.x as i32, point.y as i32);
//         let camera_position = Position::from_xy(model.camera().x as i32, model.camera().y as i32);
//         (point_position - camera_position + center) * scale
//     }

//     fn position_to_point(&self, model: &Model, position: Position) -> Point {
//         let scale = model.scale().get() as u32;
//         let center = (self.window_size / scale).to_region().center();
//         let camera_position = Position::from_xy(model.camera().x as i32, model.camera().y as i32);
//         let p = (position / scale) + camera_position - center;
//         Point::new(p.x as i16, p.y as i16)
//     }
// }
