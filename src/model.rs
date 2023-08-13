use pagurus::{
    failure::OrFail,
    image::Color,
    spatial::{Position, Region, Size},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct Model {
    cursor: Cursor,
    camera: Camera,
    dot_color: ColorIndex, // TODO: rename
    palette: Palette,
    pixels: BTreeMap<PixelPosition, ColorIndex>,
    names: BTreeMap<String, NameKind>,
    marker: Option<Marker>,
    stash_buffer: StashBuffer,
    anchors: BTreeMap<AnchorName, PixelPosition>,
    applied_commands: Vec<Command>, // dirty_commands (?)
                                    // anchors: Vec<(usize,Anchor)>
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

    pub fn take_applied_commands(&mut self) -> Vec<Command> {
        std::mem::take(&mut self.applied_commands)
    }

    pub fn applied_commands(&self) -> &[Command] {
        &self.applied_commands
    }

    pub fn has_stashed_pixels(&self) -> bool {
        !self.stash_buffer.pixels.is_empty()
    }

    pub fn stashed_pixels(&self) -> impl '_ + Iterator<Item = (PixelPosition, Color)> {
        self.stash_buffer
            .pixels
            .iter()
            .map(|(p, &color_index)| (self.cursor.position + *p, self.palette.get(color_index)))
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

    pub fn marker(&self) -> Option<&Marker> {
        self.marker.as_ref()
    }

    pub fn dot_color(&self) -> Color {
        self.palette.get(self.dot_color)
    }

    pub fn redo(&mut self, command: &Command) -> pagurus::Result<bool> {
        let applied = self.redo_command(command).or_fail()?;

        if let Some(mut marker) = self.marker.take() {
            marker.handle_command(command, self);
            self.marker = Some(marker);
        }

        Ok(applied)
    }

    fn redo_command(&mut self, command: &Command) -> pagurus::Result<bool> {
        match command {
            Command::Quit => {
                // TODO: note
                return Ok(false);
            }
            Command::Move(delta) => {
                // TODO: aggregate consecutive moves in a certain period of time into one command
                self.cursor.move_delta(*delta)
            }
            Command::Define(c) => {
                self.handle_define_command(c.0.name.clone(), c.0.value.clone())
                    .or_fail()?;
            }
            Command::Mark(kind) => {
                self.marker = Some(Marker::new(*kind, self));
                self.stash_buffer.clear();
            }
            Command::Cancel => {
                self.marker = None;
                self.stash_buffer.clear();
            }
            Command::Draw => {
                self.handle_draw_command().or_fail()?;
            }
            Command::Erase => {
                self.handle_erase_command().or_fail()?;
            }
            Command::Set(c) => {
                self.handle_set_command(c).or_fail()?;
            }
            Command::Rotate(c) => {
                self.handle_rotate_command(c).or_fail()?;
            }
            Command::Pick => {
                if let Some(color) = self.pixels.get(&self.cursor.position).copied() {
                    self.dot_color = color;
                }
            }
            Command::Cut => {
                self.handle_cut_command().or_fail()?;
            }
            Command::Paste => {
                self.handle_paste_command().or_fail()?;
            }
            Command::Anchor(c) => {
                self.handle_anchor_command(c).or_fail()?;
            }
        }
        Ok(true)
    }

    fn handle_anchor_command(&mut self, name: &AnchorName) -> pagurus::Result<()> {
        let position = self.cursor.position;
        self.anchors.insert(name.clone(), position);
        self.names.insert(name.0.clone(), NameKind::Anchor);
        Ok(())
    }

    fn handle_paste_command(&mut self) -> pagurus::Result<()> {
        let pixels = self
            .stash_buffer
            .pixels
            .iter()
            .map(|(p, &color_index)| (self.cursor.position + *p, color_index));

        for (position, color) in pixels {
            // TODO: validate whether the index exists
            self.pixels.insert(position, color);
        }
        Ok(())
    }

    fn handle_cut_command(&mut self) -> pagurus::Result<()> {
        let Some(marker) = self.marker.take() else {
            return Ok(());
        };

        self.stash_buffer.clear();
        for pixel in marker.marked_pixels() {
            if let Some(color) = self.pixels.remove(&pixel) {
                self.stash_buffer
                    .store(self.cursor.position.delta(pixel), color);
            }
        }

        Ok(())
    }

    fn handle_rotate_command(&mut self, c: &RotateCommand) -> pagurus::Result<()> {
        match c {
            RotateCommand::Color(delta) => {
                let name = self.palette.get_name(self.dot_color).or_fail()?;
                let rotated_name = if delta.0 >= 0 {
                    self.palette
                        .colors()
                        .skip_while(|c| *c != name)
                        .nth((delta.0.abs() as usize) % self.palette.len())
                        .or_fail()?
                } else {
                    self.palette
                        .colors()
                        .rev()
                        .skip_while(|c| *c != name)
                        .nth((delta.0.abs() as usize) % self.palette.len())
                        .or_fail()?
                };
                self.dot_color = self.palette.get_index(rotated_name).or_fail()?;
            }
        }
        Ok(())
    }

    fn handle_set_command(&mut self, c: &SetCommand) -> pagurus::Result<()> {
        match c {
            SetCommand::Color(name) => {
                let kind = self
                    .names
                    .get(&name.0)
                    .copied()
                    .or_fail()
                    .map_err(|f| f.message(format!("The name '{}' is not defined", name.0)))?;
                matches!(kind, NameKind::Color).or_fail().map_err(|f| {
                    f.message(format!(
                        "The name '{}' is defined as {kind} name, not a color name",
                        name.0,
                    ))
                })?;
                self.dot_color = self.palette.get_index(name).or_fail()?;
            }
            SetCommand::Cursor(name) => {
                let kind = self
                    .names
                    .get(&name.0)
                    .copied()
                    .or_fail()
                    .map_err(|f| f.message(format!("The name '{}' is not defined", name.0)))?;
                matches!(kind, NameKind::Anchor).or_fail().map_err(|f| {
                    f.message(format!(
                        "The name '{}' is defined as {kind} name, not an anchor name",
                        name.0,
                    ))
                })?;
                self.cursor.position = self.anchors.get(name).copied().or_fail()?;
            }
        }
        Ok(())
    }

    fn handle_draw_command(&mut self) -> pagurus::Result<()> {
        let Some(marker) = self.marker.take() else {
            return Ok(());
        };
        for pixel in marker.marked_pixels() {
            self.pixels.insert(pixel, self.dot_color);
        }
        Ok(())
    }

    fn handle_erase_command(&mut self) -> pagurus::Result<()> {
        let Some(marker) = self.marker.take() else {
            return Ok(());
        };
        for pixel in marker.marked_pixels() {
            self.pixels.remove(&pixel);
        }
        Ok(())
    }

    fn handle_define_command(&mut self, name: String, color: Color) -> pagurus::Result<()> {
        matches!(self.names.get(&name), None | Some(NameKind::Color))
            .or_fail()
            .map_err(|f| {
                f.message(format!(
                    "The name '{name}' is already in used by as a {} name",
                    self.names[&name]
                ))
            })?;
        self.palette.set_color(ColorName(name.clone()), color);
        self.names.insert(name, NameKind::Color);
        Ok(())
    }

    pub fn apply(&mut self, command: Command) -> pagurus::Result<()> {
        if self.redo(&command).or_fail()? {
            self.applied_commands.push(command);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NameKind {
    Color,
    Anchor,
}

impl std::fmt::Display for NameKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameKind::Color => write!(f, "a color"),
            NameKind::Anchor => write!(f, "an anchor"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, try_from = "serde_json::Value")]
pub enum CommandOrCommands {
    Commands(Vec<Command>),
    Command(Command),
}

impl CommandOrCommands {
    pub fn into_iter(self) -> impl Iterator<Item = Command> {
        match self {
            Self::Commands(commands) => commands.into_iter(),
            Self::Command(command) => vec![command].into_iter(),
        }
    }
}

impl Default for CommandOrCommands {
    fn default() -> Self {
        Self::Commands(Vec::new())
    }
}

impl TryFrom<serde_json::Value> for CommandOrCommands {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        if value.is_array() {
            serde_json::from_value(value).map(Self::Commands)
        } else {
            serde_json::from_value(value).map(Self::Command)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefineCommand(NameAndValue<Color>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "BTreeMap<String, T>", into = "BTreeMap<String, T>")]
pub struct NameAndValue<T: Clone> {
    pub name: String,
    pub value: T,
}

impl<T: Clone> From<NameAndValue<T>> for BTreeMap<String, T> {
    fn from(name_and_value: NameAndValue<T>) -> Self {
        [(name_and_value.name, name_and_value.value)]
            .into_iter()
            .collect()
    }
}

impl<T: Clone> TryFrom<BTreeMap<String, T>> for NameAndValue<T> {
    type Error = &'static str;

    fn try_from(map: BTreeMap<String, T>) -> Result<Self, Self::Error> {
        if map.len() != 1 {
            return Err("Expected exactly one name and value");
        }
        let (name, value) = map.into_iter().next().expect("unreachable");
        Ok(Self { name, value })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetCommand {
    Color(ColorName),
    Cursor(AnchorName),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RotateCommand {
    Color(RotateDelta), // TODO: Cursor
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RotateDelta(isize);

// TODO: add unit test
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    // "quit"
    Quit,

    // {"move": [0, 1]}
    Move(PixelPositionDelta),

    // {"define": {"white": [255, 255, 255]}}
    Define(DefineCommand),
    // {"rename": {"black": "white"}},
    // {"remove": "black"},
    Mark(MarkKind),
    Cancel,
    Draw,
    Erase,

    Cut,
    Paste,

    Pick,

    // {"stash": [[null, 0, 0], [1, 2, 3]]}

    // {"set": {"color": "red"}},
    // {"set": {"cursor": "anchor_name"}},
    // {"set": {"background": "red"}}
    // {"set": {"show-frame": 1}}
    // {"set": {"camera": [0, 0]}}
    Set(SetCommand),

    // {"rotate": {"color": 1}},
    Rotate(RotateCommand),

    Anchor(AnchorName),
    //---------------
    // Basic commands
    //---------------

    // {"embed": {}}
    // {"anchor": "name"}

    // {"rotate": {"color": 1}},

    // {"define": {"colors": ...}}
    // {"define": {"anchors": ...}}
    // {"define": {"frames": ...}}
    // {"rename": {"colors": ...}}
    // {"remove": {"colors": ["red", "blue"]}}
    //
    // {"tag": "foo"}
    //
    // {"embed": "frame_name"}
    // Stash(commands)
    // Embed: {"embed": {"foo": {path: "foo.de", "anchor": "name", "frames": [-1, 1, -29], "fps": 30,"position": [0,0],  "size": [100, 100]}}}
    // Mark (color)

    //
    // [0, 0] | "anchor_name" | {"anchor_name": [0, 0]}
    //
    //
    // checkpoint (chronological)
    // anchor (spatial)
    // {"anchor": {
    //     "foo": {"position": [0,10], "anchor": "origin"}
    // }
    // {"anchor": {"foo": {}}}

    //------------------
    // Compound commands
    //------------------
    // move_up = {"set": {"cursor": [0, 1]}}

    // TODO: SetDotColorByIndex

    // Cut,
    // Paste,
    // TuplePastePreview,
    // Copy = [Cut, Paste, TublePastePreview],

    // Cut = [CopyToClipboard, Erase]

    // SetClipboard(Commands)
    // ShowClipboard or preview
    // HideClipboard or unpreview
    // ClearClipboard

    // CopyToClipboard
    // StartClipboard
    // EndClipboard
    // ShowClipboard
    // PasteClipboard

    // bg or frame (iframe)
    // {"set_background": [{"color": [0,0,0], "size": [100,100]}, {"file": {"path": "path/to/image.png", "position": [0, 0], "size": [100, 100]}}]
    // animation frame
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkKind {
    Stroke,
    // Fill, SameColor, InnerEdge, OuterEdge,
    // Line, Rectangle, Ellipse
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
        self.position.x += delta.x;
        self.position.y += delta.y;
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(from = "(i16, i16)", into = "(i16, i16)")]
pub struct PixelPositionDelta {
    pub y: i16,
    pub x: i16,
}

impl PixelPositionDelta {
    pub const fn from_xy(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub const fn to_xy(self) -> (i16, i16) {
        (self.x, self.y)
    }
}

impl From<(i16, i16)> for PixelPositionDelta {
    fn from((x, y): (i16, i16)) -> Self {
        Self { x, y }
    }
}

impl From<PixelPositionDelta> for (i16, i16) {
    fn from(delta: PixelPositionDelta) -> Self {
        (delta.x, delta.y)
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct PixelPosition {
    pub y: i16,
    pub x: i16,
}

impl PixelPosition {
    pub fn delta(self, other: Self) -> PixelPositionDelta {
        PixelPositionDelta::from_xy(other.x - self.x, other.y - self.y)
    }
}

impl std::ops::Add<PixelPositionDelta> for PixelPosition {
    type Output = Self;

    fn add(self, delta: PixelPositionDelta) -> Self::Output {
        Self {
            x: self.x + delta.x,
            y: self.y + delta.y,
        }
    }
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
    // TODO: rename
    fn colors(&self) -> impl '_ + DoubleEndedIterator<Item = &ColorName> {
        self.colors.keys().chain(self.colors.keys())
    }

    fn len(&self) -> usize {
        self.colors.len()
    }

    fn set_color(&mut self, name: ColorName, color: Color) {
        if !self.table.contains(&name) {
            self.table.push(name.clone());
        }
        self.colors.insert(name, color);
    }

    fn get(&self, index: ColorIndex) -> Color {
        self.table
            .get(index.0)
            .and_then(|name| self.colors.get(name))
            .copied()
            .unwrap_or(Color::BLACK) // TODO: return Error (?)
    }

    fn get_name(&self, index: ColorIndex) -> pagurus::Result<&ColorName> {
        self.table.get(index.0).or_fail()
    }

    fn get_index(&self, color_name: &ColorName) -> pagurus::Result<ColorIndex> {
        self.colors
            .contains_key(color_name)
            .then_some(())
            .and_then(|()| self.table.iter().position(|name| name == color_name))
            .map(ColorIndex)
            .or_fail()
            .map_err(|f| f.message(format!("Color '{}' is not found", color_name.0)))
    }
}

#[derive(Debug, Clone)]
pub enum Marker {
    Stroke(StrokeMarker),
}

impl Marker {
    fn new(mark_kind: MarkKind, model: &Model) -> Self {
        match mark_kind {
            MarkKind::Stroke => Self::Stroke(StrokeMarker::new(model)),
        }
    }

    fn handle_command(&mut self, command: &Command, model: &Model) {
        match self {
            Self::Stroke(tool) => tool.handle_command(command, model),
        }
    }

    pub fn marked_pixels(&self) -> Box<dyn '_ + Iterator<Item = PixelPosition>> {
        match self {
            Self::Stroke(tool) => Box::new(tool.marked_pixels()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StrokeMarker {
    stroke: HashSet<PixelPosition>,
}

impl StrokeMarker {
    fn new(model: &Model) -> Self {
        Self {
            stroke: [model.cursor.position].into_iter().collect(),
        }
    }

    fn handle_command(&mut self, _command: &Command, model: &Model) {
        self.stroke.insert(model.cursor.position);
    }

    fn marked_pixels(&self) -> impl '_ + Iterator<Item = PixelPosition> {
        self.stroke.iter().copied()
    }
}

#[derive(Debug, Default, Clone)]
pub struct StashBuffer {
    pixels: BTreeMap<PixelPositionDelta, ColorIndex>,
}

impl StashBuffer {
    fn clear(&mut self) {
        self.pixels.clear();
    }

    fn store(&mut self, position: PixelPositionDelta, color: ColorIndex) {
        self.pixels.insert(position, color);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AnchorName(pub String);
