use pagurus::{
    failure::OrFail,
    image::Color,
    spatial::{Position, Region, Size},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
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

    pub fn visible_pixels(
        &self,
        window_size: Size,
    ) -> impl '_ + Iterator<Item = (PixelPosition, Color)> {
        let region = Region::new(
            Position::from(self.camera.position) - window_size.to_region().center(),
            window_size,
        );

        // TODO: optimize
        region.iter().filter_map(|p| {
            let pixel_position = PixelPosition::from((p.x as i16, p.y as i16));
            self.pixels
                .get(&pixel_position)
                .map(|color_index| (pixel_position, self.palette.colors[color_index]))
        })
    }

    pub fn apply(&mut self, command: ModelCommand) -> pagurus::Result<()> {
        (self.version == command.version()).or_fail().map_err(|f| {
            f.message(format!(
                "version mismatch: model version is {}, command version is {}",
                self.version.0,
                command.version().0
            ))
        })?;

        match &command {
            ModelCommand::MoveCursor { delta, .. } => self.cursor.move_delta(*delta),
            ModelCommand::Dot { .. } => {
                let old = self
                    .pixels
                    .insert(self.cursor.position, self.palette.selected);
                if old == Some(self.palette.selected) {
                    return Ok(());
                }
            }
            ModelCommand::SelectColor { index, .. } => {
                pagurus::dbg!(index);
                self.palette.colors.get(index).or_fail()?;
                self.palette.selected = *index;
                pagurus::dbg!(self.palette.selected_color());
            }
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

    pub fn dot_command(&self) -> ModelCommand {
        ModelCommand::Dot {
            version: self.version,
        }
    }
}

// TODO: remove
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct ModelVersion(pub u64);

impl ModelVersion {
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelCommand {
    MoveCursor {
        version: ModelVersion, // TODO: delete
        delta: PixelPosition,
    },
    Dot {
        version: ModelVersion,
    },
    SelectColor {
        version: ModelVersion,
        index: ColorIndex,
    },
    // Snapshot
}

impl ModelCommand {
    pub fn version(&self) -> ModelVersion {
        match self {
            ModelCommand::MoveCursor { version, .. } => *version,
            ModelCommand::Dot { version, .. } => *version,
            ModelCommand::SelectColor { version, .. } => *version,
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

impl From<PixelPosition> for Position {
    fn from(position: PixelPosition) -> Self {
        Self {
            x: position.x as i32,
            y: position.y as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColorIndex(pub usize);

#[derive(Debug, Clone)]
pub struct Palette {
    pub colors: BTreeMap<ColorIndex, Color>,
    pub selected: ColorIndex,
}

impl Palette {
    pub fn selected_color(&self) -> Color {
        self.colors[&self.selected]
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: [
                (ColorIndex(0), Color::rgb(255, 255, 255)),
                (ColorIndex(1), Color::rgb(255, 0, 0)),
                (ColorIndex(2), Color::rgb(0, 255, 0)),
                (ColorIndex(3), Color::rgb(0, 0, 255)),
                (ColorIndex(4), Color::rgb(0, 0, 0)),
            ]
            .into_iter()
            .collect(),
            selected: ColorIndex(4),
        }
    }
}
