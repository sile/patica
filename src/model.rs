use pagurus::{failure::OrFail, image::Color};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Model {
    cursor: Cursor,
    camera: Camera,
    palette: Palette,
    pixels: BTreeMap<PixelPosition, ColorIndex>,
}

impl Model {
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn camera(&self) -> Camera {
        self.camera
    }

    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    pub fn handle_command(&mut self, command: ModelCommand) -> pagurus::Result<()> {
        match command {
            ModelCommand::MoveCursor { delta } => self.handle_move_cursor_command(delta).or_fail(),
        }
    }

    fn handle_move_cursor_command(&mut self, delta: PixelPosition) -> pagurus::Result<()> {
        self.cursor.position.x += delta.x;
        self.cursor.position.y += delta.y;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ModelCommand {
    MoveCursor { delta: PixelPosition },
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Camera {
    pub position: PixelPosition,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Cursor {
    pub position: PixelPosition,
}

impl Cursor {
    pub fn move_x(&mut self, delta: i16) {
        self.position.x += delta;
    }

    pub fn move_y(&mut self, delta: i16) {
        self.position.y += delta;
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct PixelPosition {
    pub y: i16,
    pub x: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColorIndex {
    pub y: u8,
    pub x: u8,
}

impl ColorIndex {
    pub const fn from_yx(y: u8, x: u8) -> Self {
        Self { y, x }
    }
}

#[derive(Debug)]
pub struct Palette {
    pub colors: BTreeMap<ColorIndex, Color>,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: [
                (ColorIndex::from_yx(0, 0), Color::rgb(255, 255, 255)),
                (ColorIndex::from_yx(0, 1), Color::rgb(255, 0, 0)),
                (ColorIndex::from_yx(0, 2), Color::rgb(0, 255, 0)),
                (ColorIndex::from_yx(0, 3), Color::rgb(0, 0, 255)),
                (ColorIndex::from_yx(0, 4), Color::rgb(0, 0, 0)),
            ]
            .into_iter()
            .collect(),
        }
    }
}
