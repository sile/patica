use crate::{
    command::CanvasCommand,
    query::{CanvasQuery, CanvasQueryValue},
};
use orfail::OrFail;
use pati::{Color, Image, Point};

#[derive(Debug, Default)]
pub struct Canvas {
    image: Image,
    cursor: Point,
    camera: Point,
    brush_color: Color,
    background_color: Color,
}

impl Canvas {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn image(&self) -> &Image {
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

    pub fn query(&self, query: &CanvasQuery) -> CanvasQueryValue {
        match query {
            CanvasQuery::Cursor => CanvasQueryValue::Cursor(self.cursor),
            CanvasQuery::Camera => CanvasQueryValue::Camera(self.camera),
            CanvasQuery::BrushColor => CanvasQueryValue::BrushColor(self.brush_color),
            CanvasQuery::BackgroundColor => {
                CanvasQueryValue::BackgroundColor(self.background_color)
            }
        }
    }

    pub fn command(&mut self, command: &CanvasCommand) -> orfail::Result<()> {
        match command {
            CanvasCommand::Move(c) => self.handle_move(*c).or_fail()?,
        }
        Ok(())
    }

    fn handle_move(&mut self, delta: Point) -> orfail::Result<()> {
        self.cursor = self.cursor + delta;
        Ok(())
    }
}
