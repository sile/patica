use crate::{
    command::CanvasCommand,
    query::{CanvasQuery, CanvasQueryValue},
};
use orfail::OrFail;
use pati::{Color, ImageCommand, Point, VersionedImage};
use std::num::NonZeroU8;

#[derive(Debug, Default)]
pub struct Canvas {
    image: VersionedImage,
    cursor: Point,
    camera: Point,
    brush_color: Color,
    background_color: Color,
    scale: Scale,
    quit: bool,
}

impl Canvas {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn image(&self) -> &VersionedImage {
        &self.image
    }

    pub fn cursor(&self) -> Point {
        self.cursor
    }

    pub fn camera(&self) -> Point {
        self.camera
    }

    pub fn brush_color(&self) -> Color {
        self.brush_color
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }

    pub fn scale(&self) -> NonZeroU8 {
        self.scale.0
    }

    pub fn quit(&self) -> bool {
        self.quit
    }

    pub fn query(&self, query: &CanvasQuery) -> CanvasQueryValue {
        match query {
            CanvasQuery::Cursor => CanvasQueryValue::Cursor(self.cursor),
            CanvasQuery::Camera => CanvasQueryValue::Camera(self.camera),
            CanvasQuery::BrushColor => CanvasQueryValue::BrushColor(self.brush_color),
            CanvasQuery::BackgroundColor => {
                CanvasQueryValue::BackgroundColor(self.background_color)
            }
            CanvasQuery::Scale => CanvasQueryValue::Scale(self.scale.0),
        }
    }

    pub fn command(&mut self, command: &CanvasCommand) -> orfail::Result<()> {
        match command {
            CanvasCommand::Move(c) => self.handle_move(*c).or_fail()?,
            CanvasCommand::Image(c) => self.handle_image_command(c).or_fail()?,
            CanvasCommand::Scale(c) => self.handle_scale(*c).or_fail()?,
        }
        Ok(())
    }

    fn handle_move(&mut self, delta: Point) -> orfail::Result<()> {
        self.cursor = self.cursor + delta;
        Ok(())
    }

    fn handle_scale(&mut self, delta: i8) -> orfail::Result<()> {
        let scale = (self.scale.0.get() as i8 + delta).max(1).min(100);
        self.scale = Scale(NonZeroU8::new(scale as u8).expect("unreachable"));
        Ok(())
    }

    fn handle_image_command(&mut self, command: &ImageCommand) -> orfail::Result<()> {
        self.image.apply(command);
        if let ImageCommand::Put { .. } = command {
            // TODO
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct Scale(NonZeroU8);

impl Default for Scale {
    fn default() -> Self {
        Self(NonZeroU8::new(1).expect("unreachable"))
    }
}
