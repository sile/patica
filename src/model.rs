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
    active_color: ColorIndex,
    palette: Palette,
    pixels: BTreeMap<PixelPosition, ColorIndex>,
    applied_commands: Vec<Command>, // dirty_commands (?)

                                    // TODO: undo_commands: Vec<Command> or Vec<Snapshot>
}

impl Model {
    pub fn set_active_color(&mut self, name: ColorName) -> pagurus::Result<()> {
        let command = Command::Set(SetCommand::ActiveColor(name));
        self.apply(command).or_fail()?;
        Ok(())
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

    pub fn take_applied_commands(&mut self) -> Vec<Command> {
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
                .map(|&color_index| (pixel_position, self.palette.get(color_index)))
        })
    }

    pub fn active_color(&self) -> Color {
        self.palette.get(self.active_color)
    }

    pub fn apply(&mut self, command: Command) -> pagurus::Result<()> {
        match &command {
            Command::Move(delta) => {
                // TODO: aggregate consecutive moves into one command
                self.cursor.move_delta(*delta)
            }
            Command::Dot { .. } => {
                let old = self.pixels.insert(self.cursor.position, self.active_color);
                if old == Some(self.active_color) {
                    return Ok(());
                }
            }
            Command::Define(DefineCommand::Palette(colors)) => {
                self.palette = Palette::new(colors.clone()).or_fail()?;
            }
            Command::Set(SetCommand::ActiveColor(color_name)) => {
                self.active_color = self.palette.get_index(color_name).or_fail()?;
            }
        }

        self.applied_commands.push(command);

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Move(PixelPositionDelta),
    Define(DefineCommand),
    Set(SetCommand),
    Dot,
    //Pick,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefineCommand {
    Palette(BTreeMap<ColorName, Color>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetCommand {
    ActiveColor(ColorName),
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

    fn move_delta(&mut self, delta: PixelPositionDelta) {
        self.position.x += delta.x();
        self.position.y += delta.y();
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PixelPositionDelta(i16, i16);

impl PixelPositionDelta {
    pub const fn from_xy(x: i16, y: i16) -> Self {
        Self(x, y)
    }

    pub const fn to_xy(self) -> (i16, i16) {
        (self.0, self.1)
    }

    pub const fn x(self) -> i16 {
        self.0
    }

    pub const fn y(self) -> i16 {
        self.1
    }
}

impl From<(i16, i16)> for PixelPositionDelta {
    fn from((x, y): (i16, i16)) -> Self {
        Self(x, y)
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColorName(pub String);

impl From<String> for ColorName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColorIndex(pub usize);

#[derive(Debug, Default, Clone)]
pub struct Palette {
    colors: BTreeMap<ColorName, Color>,
    table: Vec<ColorName>,
}

impl Palette {
    pub fn new(colors: BTreeMap<ColorName, Color>) -> pagurus::Result<Self> {
        (!colors.is_empty()).or_fail()?;
        let table = colors.keys().cloned().collect();
        Ok(Self { colors, table })
    }

    pub fn get(&self, index: ColorIndex) -> Color {
        self.colors
            .get(&self.table[index.0])
            .copied()
            .unwrap_or(Color::BLACK)
    }

    pub fn get_index(&self, color_name: &ColorName) -> pagurus::Result<ColorIndex> {
        self.table
            .iter()
            .position(|name| name == color_name)
            .map(ColorIndex)
            .or_fail()
            .map_err(|f| f.message(format!("color '{}' not found", color_name.0)))
    }
}
