use crate::journal::JournaledModel;
use pagurus::{failure::OrFail, image::Color, spatial::Position};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    num::{NonZeroU64, NonZeroUsize},
    path::PathBuf,
};

#[derive(Debug, Default)]
pub struct Model {
    cursor: Cursor,
    camera: Camera,
    dot_color: ColorIndex, // TODO: rename
    palette: Palette,
    pixels: BTreeMap<PixelPosition, ColorIndex>,
    names: BTreeMap<String, NameKind>,
    marker: Option<Marker>,
    background: Background,
    stash_buffer: StashBuffer,
    anchors: BTreeMap<AnchorName, PixelPosition>,
    commands_len: usize,
    tags: BTreeMap<usize, Tag>,
    external_models: BTreeMap<PathBuf, JournaledModel>,
    frames: BTreeMap<FrameName, Frame>,
    applied_commands: Vec<Command>, // dirty_commands (?)
}

impl Model {
    pub fn sync_external_models(&mut self) -> pagurus::Result<()> {
        for model in self.external_models.values_mut() {
            model.sync_model().or_fail()?;
        }
        Ok(())
    }

    pub fn active_frames(&self, clock: GameClock) -> impl '_ + Iterator<Item = FramePixels> {
        self.frames
            .values()
            .filter_map(move |f| f.to_pixels_if_active(clock, self))
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

    pub fn pixels(&self) -> impl '_ + Iterator<Item = (PixelPosition, Color)> {
        self.pixels
            .iter()
            .map(move |(p, &color_index)| (*p, self.palette.get(color_index)))
    }

    pub fn get_pixel_color(&self, position: PixelPosition) -> Option<Color> {
        self.pixels
            .get(&position)
            .map(|&color_index| self.palette.get(color_index))
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
        if applied {
            self.commands_len += 1;
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
            Command::Tag(tag) => {
                self.tags.insert(self.commands_len, tag.clone());
            }
            Command::Comment(_) => {
                // Do nothing.
            }
            Command::Embed(c) => {
                self.handle_embed_command(c.0.name.clone(), c.0.value.clone())
                    .or_fail()?;
            }
        }
        Ok(true)
    }

    fn handle_embed_command(&mut self, name: String, value: Embed) -> pagurus::Result<()> {
        matches!(self.names.get(&name), None | Some(NameKind::Frame))
            .or_fail()
            .map_err(|f| {
                f.message(format!(
                    "The name '{name}' is already in used by as {} name",
                    self.names[&name]
                ))
            })?;

        let model = JournaledModel::open_if_exists(&value.path).or_fail()?;
        self.external_models.insert(value.path.clone(), model);
        let frame = Frame::new(self.cursor.position, value);
        self.frames.insert(FrameName(name.clone()), frame);

        self.names.insert(name, NameKind::Frame);
        Ok(())
    }

    fn handle_anchor_command(&mut self, name: &AnchorName) -> pagurus::Result<()> {
        matches!(self.names.get(&name.0), None | Some(NameKind::Anchor))
            .or_fail()
            .map_err(|f| {
                f.message(format!(
                    "The name '{}' is already in used by as {} name",
                    name.0, self.names[&name.0]
                ))
            })?;

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
                self.cursor.position = self.get_anchor_position(name).or_fail()?;
            }
            SetCommand::Camera(c) => match c {
                CameraPosition::Anchor(name) => {
                    self.camera.position = self.get_anchor_position(name).or_fail()?;
                }
                CameraPosition::Pixel(position) => {
                    self.camera.position = self.cursor.position + *position;
                }
            },
            SetCommand::Background(bg) => {
                self.background = bg.clone();
            }
        }
        Ok(())
    }

    pub fn background(&self) -> &Background {
        &self.background
    }

    fn get_anchor_position(&self, name: &AnchorName) -> pagurus::Result<PixelPosition> {
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
        self.anchors.get(name).copied().or_fail()
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
                    "The name '{name}' is already in used by as {} name",
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
    Frame,
}

impl std::fmt::Display for NameKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameKind::Color => write!(f, "a color"),
            NameKind::Anchor => write!(f, "an anchor"),
            NameKind::Frame => write!(f, "a frame"),
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

impl DefineCommand {
    pub fn new(name: String, color: Color) -> Self {
        Self(NameAndValue { name, value: color })
    }
}

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
    Camera(CameraPosition),
    Background(Background),
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

    // {"set": {"show-frame": 1}}
    Set(SetCommand),

    // {"rotate": {"color": 1}},
    Rotate(RotateCommand),

    Anchor(AnchorName),

    Embed(EmbedCommand),

    Tag(Tag),

    Comment(serde_json::Value),
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

    pub const fn position(self) -> PixelPosition {
        self.position
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

impl std::ops::Add for PixelPosition {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub<PixelPosition> for PixelPosition {
    type Output = PixelPositionDelta;

    fn sub(self, other: PixelPosition) -> Self::Output {
        PixelPositionDelta::from_xy(self.x - other.x, self.y - other.y)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CameraPosition {
    Anchor(AnchorName),
    Pixel(PixelPositionDelta),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PixelRegion {
    pub position: PixelPosition,
    pub size: PixelSize,
}

impl PixelRegion {
    pub fn from_corners(min_x: i16, min_y: i16, max_x: i16, max_y: i16) -> Self {
        Self {
            position: PixelPosition { x: min_x, y: min_y },
            size: PixelSize {
                width: (max_x - min_x + 1) as u16,
                height: (max_y - min_y + 1) as u16,
            },
        }
    }

    pub fn contains(self, position: PixelPosition) -> bool {
        let PixelRegion {
            position: PixelPosition { x, y },
            size: PixelSize { width, height },
        } = self;
        x <= position.x
            && position.x < x + width as i16
            && y <= position.y
            && position.y < y + height as i16
    }

    pub fn positions(self) -> impl Iterator<Item = PixelPosition> {
        (self.position.y..)
            .take(self.size.height as usize)
            .flat_map(move |y| {
                (self.position.x..)
                    .take(self.size.width as usize)
                    .map(move |x| PixelPosition { x, y })
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "(u16,u16)", from = "(u16,u16)")]
pub struct PixelSize {
    pub width: u16,
    pub height: u16,
}

impl PixelSize {
    pub fn area(self) -> u32 {
        self.width as u32 * self.height as u32
    }
}

impl From<(u16, u16)> for PixelSize {
    fn from((width, height): (u16, u16)) -> Self {
        Self { width, height }
    }
}

impl From<PixelSize> for (u16, u16) {
    fn from(size: PixelSize) -> Self {
        (size.width, size.height)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Background {
    Color(Color),
    Checkerboard(Checkerboard),
}

impl Default for Background {
    fn default() -> Self {
        Self::Checkerboard(Checkerboard::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Checkerboard {
    #[serde(default = "Checkerboard::default_dot_size")]
    pub dot_size: NonZeroUsize,

    #[serde(default = "Checkerboard::default_color1")]
    pub color1: Color,

    #[serde(default = "Checkerboard::default_color2")]
    pub color2: Color,
}

impl Checkerboard {
    fn default_dot_size() -> NonZeroUsize {
        NonZeroUsize::new(1).expect("unreachable")
    }

    fn default_color1() -> Color {
        Color::rgb(100, 100, 100)
    }

    fn default_color2() -> Color {
        Color::rgb(200, 200, 200)
    }
}

impl Default for Checkerboard {
    fn default() -> Self {
        Self {
            dot_size: Self::default_dot_size(),
            color1: Self::default_color1(),
            color2: Self::default_color2(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag(serde_json::Value);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedCommand(NameAndValue<Embed>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Embed {
    // TODO: make it possible to refer to the editing file itself.
    pub path: PathBuf,

    pub size: PixelSize,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<PixelPosition>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<AnchorName>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation: Option<Animation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Animation {
    pub timeline: Vec<FrameState>,
    pub fps: NonZeroUsize,
}

impl Default for Animation {
    fn default() -> Self {
        // Always show the frame.
        Self {
            timeline: vec![FrameState::Show(1)],
            fps: NonZeroUsize::new(1).expect("unreachable"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameState {
    Show(usize),
    Hide(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FrameName(pub String);

#[derive(Debug, Clone)]
pub struct Frame {
    pub path: PathBuf,
    pub src_region: PixelRegion,
    pub dst_position: PixelPosition,
    pub anchor: Option<AnchorName>,
    pub animation: Animation,
}

impl Frame {
    fn new(dst_position: PixelPosition, embed: Embed) -> Self {
        Self {
            path: embed.path,
            src_region: PixelRegion {
                position: embed.position.unwrap_or_default(),
                size: embed.size,
            },
            dst_position,
            anchor: embed.anchor,
            animation: embed.animation.unwrap_or_default(),
        }
    }

    fn to_pixels_if_active<'a>(
        &'a self,
        _clock: GameClock,
        model: &'a Model,
    ) -> Option<FramePixels> {
        // TODO: animation handling
        let model = model.external_models.get(&self.path)?.model(); // TODO: handle error
        Some(FramePixels { frame: self, model })
    }
}

#[derive(Debug)]
pub struct FramePixels<'a> {
    frame: &'a Frame,
    model: &'a Model,
}

impl<'a> FramePixels<'a> {
    pub fn pixels(self) -> impl 'a + Iterator<Item = (PixelPosition, Color)> {
        let mut src_region = self.frame.src_region;
        if let Some(anchor_position) = self
            .frame
            .anchor
            .as_ref()
            .and_then(|a| self.model.anchors.get(&a).copied())
        {
            src_region.position = src_region.position + anchor_position;
        } else {
            src_region.position = src_region.position + self.model.cursor().position();
        }
        let src_origin = src_region.position;
        src_region.positions().filter_map(move |p| {
            let dst_position = self.frame.dst_position + (p - src_origin);
            Some((dst_position, self.model.get_pixel_color(p)?))
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GameClock {
    pub ticks: u64,
    pub fps: NonZeroU64,
}

impl GameClock {
    pub const fn new(fps: NonZeroU64) -> Self {
        Self { ticks: 0, fps }
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
    }
}

impl Default for GameClock {
    fn default() -> Self {
        Self::new(NonZeroU64::new(30).expect("unreachable"))
    }
}
