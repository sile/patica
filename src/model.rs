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
    dot_color: ColorIndex,
    palette: Palette,
    pixels: BTreeMap<PixelPosition, ColorIndex>,
    active_tool: Option<Tool>,
    applied_commands: Vec<Command>, // dirty_commands (?)
                                    // anchors: Vec<(usize,Anchor)>
}

impl Model {
    pub fn set_dot_color(&mut self, name: ColorName) -> pagurus::Result<()> {
        self.apply(Command::SetDotColor(name)).or_fail()?;
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
        std::mem::take(&mut self.applied_commands)
    }

    pub fn applied_commands(&self) -> &[Command] {
        &self.applied_commands
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

    pub fn active_tool(&self) -> Option<&Tool> {
        self.active_tool.as_ref()
    }

    pub fn dot_color(&self) -> Color {
        self.palette.get(self.dot_color)
    }

    pub fn redo(&mut self, command: &Command) -> pagurus::Result<bool> {
        let applied = self.redo_command(command).or_fail()?;
        if let Some(mut tool) = self.active_tool.take() {
            tool.handle_command(command, self);
            self.active_tool = Some(tool);
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
            Command::Dot => {
                let old = self.pixels.insert(self.cursor.position, self.dot_color);
                return Ok(old != Some(self.dot_color));
            }
            Command::Define(colors) => {
                let removed_indices = self.palette.handle_define_command(colors.clone());
                for index in removed_indices {
                    self.pixels
                        .retain(|_, &mut color_index| color_index != index);
                }
            }
            Command::SetDotColor(color_name) => {
                self.dot_color = self.palette.get_index(&color_name).or_fail()?;
            }
            Command::Anchor(_) => {
                // Do nothing
            }
            Command::ActivateDrawTool(mark) => {
                self.active_tool = Some(Tool::new(*mark, self));
            }
            Command::FixTool => {
                let Some(tool) = self.active_tool.take() else {
                    return Ok(false);
                };
                for position in tool.marked_pixels() {
                    match tool {
                        Tool::Stroke(_) => {
                            self.pixels.insert(position, self.dot_color);
                        }
                    }
                }
            }
            Command::CancelTool => {
                self.active_tool = None;
            }
        }
        Ok(true)
    }

    pub fn apply(&mut self, command: Command) -> pagurus::Result<()> {
        if self.redo(&command).or_fail()? {
            self.applied_commands.push(command);
        }
        Ok(())
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

// TODO: add unit test
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Quit,
    Move(PixelPositionDelta),

    //--------
    // Palette
    //--------
    Define(BTreeMap<ColorName, Option<Color>>),

    //-------
    // Marker
    //-------

    //---------------
    // Basic commands
    //---------------

    // {"rename": {"black": "white"}},
    // {"remove": "black"},
    // {"define": {"name": "black", "color": [0, 0, 0]}},
    // {"update": {"name": "black", "color": [0, 0, 0]}},
    // {"define": {"name": "foo", "frame": ""}},
    // {"define": {"name": "place", "allow_update": true, "anchor": {}}},
    // {"define": {"black: {"type":"color", "update_if_exists":true, "value":[0, 0, 0]}}},
    // {"update": {"black": [0, 0, 0]}},

    // {"move": [0, 1]}
    // {"set": {"cursor": "anchor_name"}}

    // {"set": {"color": "red"}},
    // {"rotate": {"color": 1}},
    // {"set": {"color": "black"}},

    // {"define": {"colors": ...}}
    // {"define": {"anchors": ...}}
    // {"define": {"frames": ...}}
    // {"rename": {"colors": ...}}
    // {"remove": {"colors": ["red", "blue"]}}
    //
    // {"tag": "foo"}
    //
    // Draw
    // Erase
    // Cut // or kill
    // Copy = [Cut, Paste], {"apply": ["cut", "paste"]}
    // Paste // or yank
    // Stash(commands)
    // Embed: {"embed": {"foo": {path: "foo.de", "anchor": "name", "frames": [-1, 1, -29], "fps": 30,"position": [0,0],  "size": [100, 100]}}}
    // Mark (color)
    // Cancel (mark or clipboard)

    // {"set": {"camera": [0, 0]}}
    // {"set": {"camera": "foo"}}
    //
    // [0, 0] | "anchor_name" | {"anchor_name": [0, 0]}
    //
    // {"set": {"color": "red"}}
    // {"set": {"cursor": "name"}}
    // {"set": {"background": "red"}}
    // {"rotate": {"color": 1}}
    // {"rotate": {"show-embedding": 1}}
    // {"rotate": {"cursor": 0}}
    // {"rotate": {"camera": 1}}
    //
    // Select (color) => set
    // Set / unset: show-embedding
    // toggle or shift or rotate

    // pick
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
    SetDotColor(ColorName),
    // TODO: SetDotColorByIndex
    ActivateDrawTool(MarkKind),
    FixTool,
    CancelTool,

    // Activate(Tool),
    // Deactivate,

    // MarkStart, (mark or select)
    // MarkFix,
    // MarkCancel
    // MoveInnerEdges, OuterEdges (mark ?)
    Dot, // [StartMark, Draw, FixMark]
    // Undot
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

    //Pick,
    //Quit,
    Anchor(serde_json::Value), // TODO: Add timestamp field (?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkKind {
    Stroke,
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
    fn handle_define_command(
        &mut self,
        colors: BTreeMap<ColorName, Option<Color>>,
    ) -> Vec<ColorIndex> {
        let mut removed = Vec::new();
        for (i, (name, color)) in colors.into_iter().enumerate() {
            if let Some(color) = color {
                self.colors.insert(name.clone(), color);
                if !self.table.contains(&name) {
                    self.table.push(name);
                }
            } else {
                if self.colors.remove(&name).is_some() {
                    removed.push(ColorIndex(i));
                }
                // NOTE: Don't remove the color name from self.table so that keep color indices unchanged.
            }
        }
        removed
    }

    fn get(&self, index: ColorIndex) -> Color {
        self.table
            .get(index.0)
            .and_then(|name| self.colors.get(name))
            .copied()
            .unwrap_or(Color::BLACK) // TODO: return Error (?)
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
pub enum Tool {
    Stroke(StrokeTool),
}

impl Tool {
    fn new(mark_kind: MarkKind, model: &Model) -> Self {
        match mark_kind {
            MarkKind::Stroke => Self::Stroke(StrokeTool::new(model)),
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
pub struct StrokeTool {
    stroke: HashSet<PixelPosition>,
}

impl StrokeTool {
    fn new(model: &Model) -> Self {
        Self {
            stroke: vec![model.cursor.position].into_iter().collect(),
        }
    }

    fn handle_command(&mut self, _command: &Command, model: &Model) {
        self.stroke.insert(model.cursor.position);
    }

    fn marked_pixels(&self) -> impl '_ + Iterator<Item = PixelPosition> {
        self.stroke.iter().copied()
    }
}
