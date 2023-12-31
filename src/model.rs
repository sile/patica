use orfail::OrFail;
use paticanvas::{CanvasCommand, CanvasFile, CanvasQuery, CanvasQueryValue};
use std::num::NonZeroU8;

#[derive(Debug)]
pub struct Model(CanvasFile);

impl Model {
    pub fn new(canvas_file: CanvasFile) -> Self {
        Self(canvas_file)
    }

    pub fn command(&mut self, command: &CanvasCommand) -> orfail::Result<()> {
        self.0.command(command).or_fail()
    }

    pub fn query(&self, query: &CanvasQuery) -> CanvasQueryValue {
        self.0.canvas().query(query)
    }

    pub fn sync(&mut self) -> orfail::Result<()> {
        self.0.sync().or_fail()
    }

    pub fn fps(&self) -> NonZeroU8 {
        self.0.canvas().fps()
    }

    pub fn quit(&self) -> bool {
        self.0.canvas().quit()
    }
}

// use crate::{
//     clock::{Ticks, Time},
//     command::{
//         CenterPoint, Command, ExternalCommand, FlipDirection, MoveDestination, PlayCommand,
//         RemoveTarget,
//     },
//     editor::Editor,
//     frame::{EmbeddedFrame, Frame},
//     marker::{MarkKind, Marker},
//     query::Query,
// };
// use orfail::OrFail;
// use pati::{Color, Point, Version};
// use std::{collections::BTreeMap, num::NonZeroUsize};

// const METADATA_BACKGROUND_COLOR: &str = "patica.background_color";
// const METADATA_FRAME_PREFIX: &str = "patica.frame.";
// const METADATA_BRUSH_COLOR: &str = "patica.brush_color";

// #[derive(Debug, Default)]
// pub struct Model {
//     canvas: pati::VersionedImage,
//     cursor: Point,
//     camera: Point,
//     brush_color: Color,
//     background_color: Color,
//     quit: bool,
//     fsm: Fsm,
//     scale: Scale,
//     frames: BTreeMap<String, EmbeddedFrame>,
//     ticks: Ticks,
// }

// impl Model {
//     pub fn initialize(&mut self) -> orfail::Result<()> {
//         // Background color.
//         if let Some(color_json) = self.canvas.metadata().get(METADATA_BACKGROUND_COLOR) {
//             let color = serde_json::from_value(color_json.clone()).or_fail()?;
//             self.background_color = color;
//         }

//         // Brush color.
//         if let Some(color_json) = self.canvas.metadata().get(METADATA_BRUSH_COLOR) {
//             let color = serde_json::from_value(color_json.clone()).or_fail()?;
//             self.brush_color = color;
//         }

//         // Frames.
//         for (name, value) in self.canvas.metadata() {
//             if !name.starts_with(METADATA_FRAME_PREFIX) {
//                 continue;
//             }

//             let frame = serde_json::from_value::<EmbeddedFrame>(value.clone()).or_fail()?;
//             self.frames.insert(frame.frame.name.clone(), frame);
//         }

//         Ok(())
//     }

//     pub fn cursor(&self) -> Point {
//         self.cursor
//     }

//     pub fn camera(&self) -> Point {
//         self.camera
//     }

//     pub fn brush_color(&self) -> Color {
//         self.brush_color
//     }

//     pub fn background_color(&self) -> Color {
//         self.background_color
//     }

//     pub fn frames(&self) -> &BTreeMap<String, EmbeddedFrame> {
//         &self.frames
//     }

//     pub fn frames_mut(&mut self) -> &mut BTreeMap<String, EmbeddedFrame> {
//         &mut self.frames
//     }

//     pub fn quit(&self) -> bool {
//         self.quit
//     }

//     pub fn ticks(&self) -> Ticks {
//         self.ticks
//     }

//     pub fn tick(&mut self) {
//         if let Fsm::Playing { end_time, repeat } = &self.fsm {
//             if self.ticks >= end_time.ticks {
//                 if *repeat {
//                     self.ticks = Ticks::new(0);
//                 } else {
//                     self.fsm = Fsm::Neutral(Default::default());
//                 }
//             } else {
//                 self.ticks.tick();
//             }
//         }
//     }

//     pub fn fps(&self) -> u32 {
//         if let Fsm::Playing { end_time, .. } = &self.fsm {
//             end_time.fps.get() as u32
//         } else {
//             Time::DEFAULT_FPS as u32
//         }
//     }

//     pub fn scale(&self) -> NonZeroUsize {
//         self.scale.0
//     }

//     pub fn marker(&self) -> Option<&Marker> {
//         if let Fsm::Marking(marker) = &self.fsm {
//             Some(marker)
//         } else {
//             None
//         }
//     }

//     pub fn editor(&self) -> Option<&Editor> {
//         if let Fsm::Editing(editor) = &self.fsm {
//             Some(editor)
//         } else {
//             None
//         }
//     }

//     pub fn canvas(&self) -> &pati::VersionedImage {
//         &self.canvas
//     }

//     pub fn canvas_mut(&mut self) -> &mut pati::VersionedImage {
//         &mut self.canvas
//     }

//     pub fn apply(&mut self, command: &Command) {
//         match command {
//             Command::Move(c) => self.handle_move_command(c),
//             Command::Mark(c) => self.handle_mark_command(*c),
//             Command::Pick => self.handle_pick_command(),
//             Command::Cut => self.handle_cut_command(),
//             Command::Copy => self.handle_copy_command(),
//             Command::Cancel => self.handle_cancel_command(),
//             Command::Erase => self.handle_erase_command(),
//             Command::Draw => self.handle_draw_command(),
//             Command::Undo => self.handle_undo_command(),
//             Command::Redo => self.handle_redo_command(),
//             Command::Quit => {
//                 self.quit = true;
//             }
//             Command::Dip(c) => self.handle_dip_command(*c),
//             Command::Scale(c) => self.handle_scale_command(*c),
//             Command::Center(c) => self.handle_center_command(c),
//             Command::Anchor(c) => self.handle_anchor_command(c),
//             Command::BackgroundColor(c) => self.handle_background_color_command(*c),
//             Command::Import(c) => self.handle_import_command(c),
//             Command::Embed(c) => self.handle_embed_command(c),
//             Command::Tick(c) => self.handle_tick_command(*c),
//             Command::Play(c) => self.handle_play_command(c),
//             Command::Remove(c) => self.handle_remove_command(c),
//             Command::Color(c) => self.handle_color_command(*c),
//             Command::Flip(c) => self.handle_flip_command(*c),
//             Command::Rotate => self.handle_rotate_command(),
//             Command::ExternalCommand(c) => self.handle_external_command(c),
//         }
//     }

//     pub fn query(&self, query: &Query) -> Option<serde_json::Value> {
//         match query {
//             Query::BackgroundColor => serde_json::to_value(self.background_color).ok(),
//             Query::BrushColor => serde_json::to_value(self.brush_color).ok(),
//         }
//     }

//     fn handle_external_command(&mut self, command: &ExternalCommand) {
//         let mut args = Vec::new();
//         for arg in &command.args {
//             // TODO: error handling
//             args.push(arg.try_to_string(self).unwrap_or_default());
//         }

//         // TODO: error handling
//         let _ = std::process::Command::new(&command.program)
//             .args(&args)
//             .output();
//     }

//     fn handle_rotate_command(&mut self) {
//         // TODO: Enter editor mode if not in it.
//         if let Fsm::Editing(editor) = &mut self.fsm {
//             editor.apply_rotate();
//         }
//     }

//     fn handle_flip_command(&mut self, direction: FlipDirection) {
//         // TODO: Enter editor mode if not in it.
//         if let Fsm::Editing(editor) = &mut self.fsm {
//             editor.apply_flip(direction);
//         }
//     }

//     fn handle_color_command(&mut self, color: Color) {
//         // TODO: Enter editor mode if not in it.
//         if let Fsm::Editing(editor) = &mut self.fsm {
//             editor.apply_color(color);
//         }
//     }

//     fn handle_remove_command(&mut self, target: &RemoveTarget) {
//         match target {
//             RemoveTarget::Anchor(name) => {
//                 self.canvas
//                     .apply(&pati::ImageCommand::anchor(name.clone(), None));
//             }
//             RemoveTarget::Frame(name) => {
//                 if self.frames.remove(name).is_some() {
//                     self.canvas.apply(&pati::ImageCommand::put(
//                         format!("{}{}", METADATA_FRAME_PREFIX, name),
//                         serde_json::Value::Null,
//                     ));
//                 }
//             }
//         }
//     }

//     fn handle_play_command(&mut self, command: &PlayCommand) {
//         self.ticks = command.offset;
//         self.fsm = Fsm::Playing {
//             end_time: Time::new(command.duration, command.fps),
//             repeat: command.repeat,
//         };
//     }

//     fn handle_tick_command(&mut self, delta: i32) {
//         self.ticks.tick_delta(delta);
//     }

//     fn handle_embed_command(&mut self, frame: &Frame) {
//         let frame = EmbeddedFrame::new(frame.clone(), self.cursor);
//         self.frames.insert(frame.frame.name.clone(), frame.clone());
//         let command = pati::ImageCommand::put(
//             format!("{}{}", METADATA_FRAME_PREFIX, frame.frame.name),
//             serde_json::to_value(&frame).expect("unreachable"),
//         );
//         self.canvas.apply(&command);
//     }

//     fn handle_import_command(&mut self, pixels: &[(Point, Color)]) {
//         self.fsm = Fsm::Editing(Editor::new(pixels.iter().cloned().collect()));
//     }

//     fn handle_background_color_command(&mut self, color: Color) {
//         if self.background_color != color {
//             self.background_color = color;
//             let command = pati::ImageCommand::put(
//                 METADATA_BACKGROUND_COLOR.to_owned(),
//                 serde_json::to_value(color).expect("unreachable"),
//             );
//             self.canvas.apply(&command);
//         }
//     }

//     fn handle_anchor_command(&mut self, name: &str) {
//         let command = pati::ImageCommand::anchor(name.to_owned(), Some(self.cursor));
//         self.canvas.apply(&command);
//     }

//     fn handle_center_command(&mut self, point: &CenterPoint) {
//         match point {
//             CenterPoint::Cursor => {
//                 self.camera = self.cursor;
//             }
//             CenterPoint::Anchor(name) => {
//                 if let Some(point) = self.canvas.anchors().get(name).copied() {
//                     self.camera = point;
//                 }
//             }
//         }
//     }

//     fn handle_scale_command(&mut self, delta: i8) {
//         self.scale = self.scale.saturating_add(delta);
//     }

//     fn handle_copy_command(&mut self) {
//         let Fsm::Marking(marker) = &mut self.fsm else {
//             return;
//         };

//         let mut pixels = BTreeMap::new();
//         for point in marker.marked_points() {
//             if let Some(color) = self.canvas.get_pixel(point) {
//                 pixels.insert(point - self.cursor, color);
//             }
//         }

//         self.fsm = Fsm::Editing(Editor::new(pixels));
//     }

//     fn handle_cut_command(&mut self) {
//         let Fsm::Marking(marker) = &mut self.fsm else {
//             return;
//         };

//         let mut pixels = BTreeMap::new();
//         let mut points = Vec::new();
//         for point in marker.marked_points() {
//             points.push(point);
//             if let Some(color) = self.canvas.get_pixel(point) {
//                 pixels.insert(point - self.cursor, color);
//             }
//         }

//         let command =
//             pati::ImageCommand::Patch(pati::PatchImageCommand::new(vec![pati::PatchEntry {
//                 color: None,
//                 points,
//             }]));
//         self.canvas.apply(&command);

//         self.fsm = Fsm::Editing(Editor::new(pixels));
//     }

//     fn handle_undo_command(&mut self) {
//         if let Fsm::Neutral(fsm) = &mut self.fsm {
//             let mut undo = fsm.undo.unwrap_or_else(|| Undo::new(self.canvas.version()));
//             if let Some(command) = self.canvas.diff(undo.undo_version) {
//                 self.canvas.apply(&pati::ImageCommand::Patch(command));
//                 undo.undo_version = undo.undo_version - 1;
//                 fsm.undo = Some(undo);
//             }
//         }
//     }

//     fn handle_redo_command(&mut self) {
//         if let Fsm::Neutral(fsm) = &mut self.fsm {
//             if let Some(mut undo) = fsm.undo.take() {
//                 undo.undo_version = undo.undo_version + 1;
//                 let version = undo.undo_version + 1;
//                 if let Some(command) = self.canvas.diff(version) {
//                     self.canvas.apply(&pati::ImageCommand::Patch(command));
//                     if version < undo.latest_version {
//                         fsm.undo = Some(undo);
//                     }
//                 }
//             }
//         }
//     }

//     fn handle_cancel_command(&mut self) {
//         self.fsm = Fsm::Neutral(Default::default());
//     }

//     fn handle_dip_command(&mut self, color: Color) {
//         self.set_brush_color(color);
//     }

//     fn set_brush_color(&mut self, color: Color) {
//         self.brush_color = color;
//         let command = pati::ImageCommand::put(
//             METADATA_BRUSH_COLOR.to_owned(),
//             serde_json::to_value(color).expect("unreachable"),
//         );
//         self.canvas.apply(&command);
//     }

//     fn handle_pick_command(&mut self) {
//         let cursor = self.cursor;
//         if let Some(color) = self.canvas.get_pixel(cursor) {
//             self.set_brush_color(color);
//         } else {
//             let ticks = self.ticks;
//             if let Some(color) = self
//                 .frames
//                 .values()
//                 .rev()
//                 .filter(|f| f.frame.is_visible(ticks))
//                 .find_map(|f| f.pixels.get(&cursor).copied())
//             {
//                 self.set_brush_color(color);
//             }
//         };
//     }

//     fn handle_erase_command(&mut self) {
//         let points = match &self.fsm {
//             Fsm::Neutral(_) => vec![self.cursor],
//             Fsm::Marking(marker) => marker.marked_points().collect(),
//             Fsm::Editing(_) | Fsm::Playing { .. } => Vec::new(),
//         };
//         let command =
//             pati::ImageCommand::Patch(pati::PatchImageCommand::new(vec![pati::PatchEntry {
//                 color: None,
//                 points,
//             }]));
//         self.canvas.apply(&command);
//         self.fsm = Fsm::Neutral(Default::default());
//     }

//     fn handle_draw_command(&mut self) {
//         self.fsm
//             .draw(&mut self.canvas, self.brush_color, self.cursor);
//     }

//     fn handle_mark_command(&mut self, kind: MarkKind) {
//         self.fsm = Fsm::Marking(Marker::new(kind, self));
//     }

//     fn handle_move_command(&mut self, dst: &MoveDestination) {
//         match dst {
//             MoveDestination::Delta(delta) => {
//                 self.cursor = self.cursor + *delta;
//             }
//             MoveDestination::Anchor(name) => {
//                 if let Some(point) = self.canvas.anchors().get(&name.anchor).copied() {
//                     self.cursor = point;
//                 }
//             }
//         }
//         if matches!(self.fsm, Fsm::Marking(_)) {
//             if let Fsm::Marking(mut marker) = std::mem::take(&mut self.fsm) {
//                 marker.handle_move(self);
//                 self.fsm = Fsm::Marking(marker);
//             }
//         }
//     }
// }

// #[derive(Debug)]
// enum Fsm {
//     Neutral(NeutralState),
//     Marking(Marker),
//     Editing(Editor),
//     Playing { end_time: Time, repeat: bool },
// }

// impl Fsm {
//     fn draw(&mut self, canvas: &mut pati::VersionedImage, brush_color: Color, cursor: Point) {
//         match self {
//             Fsm::Neutral(fsm) => {
//                 let command = pati::ImageCommand::patch(vec![pati::PatchEntry::draw(
//                     brush_color,
//                     vec![cursor],
//                 )]);
//                 canvas.apply(&command);
//                 fsm.undo = None;
//             }
//             Fsm::Marking(fsm) => {
//                 let command = pati::ImageCommand::patch(vec![pati::PatchEntry::draw(
//                     brush_color,
//                     fsm.marked_points().collect(),
//                 )]);
//                 canvas.apply(&command);
//             }
//             Fsm::Editing(fsm) => {
//                 let mut patches: BTreeMap<_, pati::PatchEntry> = BTreeMap::new();
//                 for (point, color) in fsm.pixels() {
//                     patches
//                         .entry(color)
//                         .or_insert_with(|| pati::PatchEntry::draw(color, Vec::new()))
//                         .points
//                         .push(point + cursor);
//                 }
//                 let command = pati::ImageCommand::patch(patches.into_values().collect());
//                 canvas.apply(&command);
//             }
//             Fsm::Playing { .. } => {}
//         }
//     }
// }

// impl Default for Fsm {
//     fn default() -> Self {
//         Self::Neutral(NeutralState::default())
//     }
// }

// #[derive(Debug, Clone, Copy)]
// struct Undo {
//     latest_version: Version,
//     undo_version: Version,
// }

// impl Undo {
//     fn new(version: Version) -> Self {
//         Self {
//             latest_version: version,
//             undo_version: version - 1,
//         }
//     }
// }

// #[derive(Debug, Default)]
// struct NeutralState {
//     undo: Option<Undo>,
// }

// #[derive(Debug, Clone, Copy)]
// struct Scale(NonZeroUsize);

// impl Scale {
//     fn saturating_add(self, delta: i8) -> Self {
//         let n = (self.0.get() as i8 + delta).max(1).min(100);
//         Self(NonZeroUsize::new(n as usize).expect("unreachable"))
//     }
// }

// impl Default for Scale {
//     fn default() -> Self {
//         Self(NonZeroUsize::new(1).expect("unreachable"))
//     }
// }
