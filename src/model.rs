use pagurus::{
    failure::OrFail,
    image::Color,
    spatial::{Position, Region, Size},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub struct Model {
    cursor: Cursor,
    camera: Camera,
    palette: Palette,
    pixels: BTreeMap<PixelPosition, ColorIndex>,
    applied_commands: Vec<ModelCommand>,
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

        Ok(())
    }

    pub fn move_cursor_command(&self, delta: PixelPosition) -> ModelCommand {
        ModelCommand::MoveCursor { delta }
    }

    pub fn dot_command(&self) -> ModelCommand {
        ModelCommand::Dot {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelCommand {
    MoveCursor { delta: PixelPosition },
    Dot {},
    SelectColor { index: ColorIndex },
    // Snapshot
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
