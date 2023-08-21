use crate::{color::Color, spatial::CanvasPosition};

#[derive(Debug, Clone)]
pub struct Canvas {
    cursor: CanvasPosition,
    brush_color: Color,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            cursor: CanvasPosition::default(),
            brush_color: Color::rgb(0, 0, 0),
        }
    }

    pub fn cursor(&self) -> CanvasPosition {
        self.cursor
    }

    pub fn brush_color(&self) -> Color {
        self.brush_color
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}
