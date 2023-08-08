use pagurus::{failure::OrFail, image::Color};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Model {
    version: ModelVersion,
    cursor: Cursor,
    camera: Camera,
    palette: Palette,
    pixels: BTreeMap<PixelPosition, ColorIndex>,
    applied_commands: Vec<ModelCommand>,
}

impl Model {
    pub fn version(&self) -> ModelVersion {
        self.version
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn camera(&self) -> Camera {
        self.camera
    }

    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    pub fn take_applied_commands(&mut self) -> Vec<ModelCommand> {
        // TODO: compaction
        std::mem::take(&mut self.applied_commands)
    }

    pub fn apply(&mut self, command: ModelCommand) -> pagurus::Result<()> {
        (self.version == command.version()).or_fail()?;

        match &command {
            ModelCommand::MoveCursor { delta, .. } => self.cursor.move_delta(*delta),
        }

        self.applied_commands.push(command);
        self.version.0 += 1;

        Ok(())
    }

    pub fn move_cursor_command(&self, delta: PixelPosition) -> ModelCommand {
        ModelCommand::MoveCursor {
            version: self.version,
            delta,
        }
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct ModelVersion(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelCommand {
    MoveCursor {
        version: ModelVersion,
        delta: PixelPosition,
    },
}

impl ModelCommand {
    pub fn version(&self) -> ModelVersion {
        match self {
            ModelCommand::MoveCursor { version, .. } => *version,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Camera {
    pub position: PixelPosition,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Cursor {
    position: PixelPosition,
}

impl Cursor {
    pub const fn x(self) -> i16 {
        self.position.x
    }

    pub const fn y(self) -> i16 {
        self.position.y
    }

    pub fn move_x(&mut self, delta: i16) {
        self.position.x += delta;
    }

    pub fn move_y(&mut self, delta: i16) {
        self.position.y += delta;
    }

    fn move_delta(&mut self, delta: PixelPosition) {
        self.position.x += delta.x;
        self.position.y += delta.y;
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct PixelPosition {
    pub y: i16,
    pub x: i16,
}

impl From<(i16, i16)> for PixelPosition {
    fn from((x, y): (i16, i16)) -> Self {
        Self { x, y }
    }
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
