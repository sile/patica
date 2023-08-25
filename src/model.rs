use crate::{
    clock::Clock,
    command::{Command, MoveDestination},
    editor::Editor,
    marker::{MarkKind, Marker},
};
use pati::{Color, Point, Version};
use std::{collections::BTreeMap, num::NonZeroUsize};

#[derive(Debug, Default)]
pub struct Model {
    canvas: pati::VersionedCanvas,
    cursor: Point,
    camera: Point,
    brush_color: Color,
    clock: Clock,
    quit: bool,
    fsm: Fsm,
    scale: Scale,
}

impl Model {
    pub fn cursor(&self) -> Point {
        self.cursor
    }

    pub fn camera(&self) -> Point {
        self.camera
    }

    pub fn brush_color(&self) -> Color {
        self.brush_color
    }

    pub fn quit(&self) -> bool {
        self.quit
    }

    pub fn clock(&self) -> Clock {
        self.clock
    }

    pub fn tick(&mut self) {
        self.clock.tick();
    }

    pub fn scale(&self) -> NonZeroUsize {
        self.scale.0
    }

    pub fn marker(&self) -> Option<&Marker> {
        if let Fsm::Marking(marker) = &self.fsm {
            Some(marker)
        } else {
            None
        }
    }

    pub fn editor(&self) -> Option<&Editor> {
        if let Fsm::Editing(editor) = &self.fsm {
            Some(editor)
        } else {
            None
        }
    }

    pub fn canvas(&self) -> &pati::VersionedCanvas {
        &self.canvas
    }

    pub fn canvas_mut(&mut self) -> &mut pati::VersionedCanvas {
        &mut self.canvas
    }

    pub fn apply(&mut self, command: &Command) {
        match command {
            Command::Move(c) => self.handle_move_command(c),
            Command::Mark(c) => self.handle_mark_command(*c),
            Command::Pick => self.handle_pick_command(),
            Command::Cut => self.handle_cut_command(),
            Command::Cancel => self.handle_cancel_command(),
            Command::Erase => self.handle_erase_command(),
            Command::Color => self.handle_color_command(),
            Command::Paste => self.handle_paste_command(),
            Command::Undo => self.handle_undo_command(),
            Command::Redo => self.handle_redo_command(),
            Command::Quit => {
                self.quit = true;
            }
            Command::Dip(c) => self.handle_dip_command(*c),
            Command::Scale(c) => self.handle_scale_command(*c),
        }
    }

    fn handle_scale_command(&mut self, delta: i8) {
        self.scale = self.scale.saturating_add(delta);
    }

    fn handle_paste_command(&mut self) {
        let Fsm::Editing(editor) = &self.fsm else {
            return;
        };

        let mut pixels: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for (point, color) in editor.pixels() {
            pixels.entry(color).or_default().push(point + self.cursor);
        }

        let entries = pixels
            .into_iter()
            .map(|(color, points)| pati::PatchEntry {
                color: Some(color),
                points,
            })
            .collect::<Vec<_>>();
        let command = pati::Command::Patch(pati::PatchCommand::new(entries));
        self.canvas.apply(&command);
    }

    fn handle_cut_command(&mut self) {
        let Fsm::Marking(marker) = &mut self.fsm else {
            return;
        };

        let mut pixels = BTreeMap::new();
        let mut points = Vec::new();
        for point in marker.marked_points() {
            points.push(point);
            if let Some(color) = self.canvas.get_pixel(point) {
                pixels.insert(point - self.cursor, color);
            }
        }

        let command = pati::Command::Patch(pati::PatchCommand::new(vec![pati::PatchEntry {
            color: None,
            points,
        }]));
        self.canvas.apply(&command);

        self.fsm = Fsm::Editing(Editor::new(pixels));
    }

    fn handle_undo_command(&mut self) {
        if let Fsm::Neutral(fsm) = &mut self.fsm {
            let mut undo = fsm.undo.unwrap_or_else(|| Undo::new(self.canvas.version()));
            if let Some(command) = self.canvas.diff(undo.undo_version) {
                self.canvas.apply(&pati::Command::Patch(command));
                undo.undo_version = undo.undo_version - 1;
                fsm.undo = Some(undo);
            }
        }
    }

    fn handle_redo_command(&mut self) {
        if let Fsm::Neutral(fsm) = &mut self.fsm {
            if let Some(mut undo) = fsm.undo.take() {
                undo.undo_version = undo.undo_version + 1;
                let version = undo.undo_version + 1;
                if let Some(command) = self.canvas.diff(version) {
                    self.canvas.apply(&pati::Command::Patch(command));
                    if version < undo.latest_version {
                        fsm.undo = Some(undo);
                    }
                }
            }
        }
    }

    fn handle_cancel_command(&mut self) {
        self.fsm = Fsm::Neutral(Default::default());
    }

    fn handle_dip_command(&mut self, color: Color) {
        self.brush_color = color;
    }

    fn handle_pick_command(&mut self) {
        if let Some(color) = self.get_displayed_pixel_color(self.cursor) {
            self.brush_color = color;
        }
    }

    fn get_displayed_pixel_color(&self, point: Point) -> Option<Color> {
        // TODO: consider background
        self.canvas.get_pixel(point)
    }

    fn handle_erase_command(&mut self) {
        let points = match &self.fsm {
            Fsm::Neutral(_) => vec![self.cursor],
            Fsm::Marking(marker) => marker.marked_points().collect(),
            Fsm::Editing(_) => Vec::new(),
        };
        let command = pati::Command::Patch(pati::PatchCommand::new(vec![pati::PatchEntry {
            color: None,
            points,
        }]));
        self.canvas.apply(&command);
        self.fsm = Fsm::Neutral(Default::default());
    }

    fn handle_color_command(&mut self) {
        let points = match &self.fsm {
            Fsm::Neutral(_) => vec![self.cursor],
            Fsm::Marking(marker) => marker.marked_points().collect(),
            Fsm::Editing(_) => vec![],
        };
        let command = pati::Command::Patch(pati::PatchCommand::new(vec![pati::PatchEntry {
            color: Some(self.brush_color),
            points,
        }]));
        self.canvas.apply(&command);
        self.fsm = Fsm::Neutral(Default::default());
    }

    fn handle_mark_command(&mut self, kind: MarkKind) {
        self.fsm = Fsm::Marking(Marker::new(kind, self.cursor));
    }

    fn handle_move_command(&mut self, dst: &MoveDestination) {
        match dst {
            MoveDestination::Delta(delta) => {
                self.cursor = self.cursor + *delta;
            }
            MoveDestination::Anchor(_) => todo!(),
        }
        if let Fsm::Marking(marker) = &mut self.fsm {
            marker.handle_move(self.cursor);
        }
    }
}

#[derive(Debug)]
enum Fsm {
    Neutral(NeutralState),
    Marking(Marker),
    Editing(Editor),
}

impl Default for Fsm {
    fn default() -> Self {
        Self::Neutral(NeutralState::default())
    }
}

#[derive(Debug, Clone, Copy)]
struct Undo {
    latest_version: Version,
    undo_version: Version,
}

impl Undo {
    fn new(version: Version) -> Self {
        Self {
            latest_version: version,
            undo_version: version - 1,
        }
    }
}

#[derive(Debug, Default)]
struct NeutralState {
    undo: Option<Undo>,
}

#[derive(Debug, Clone, Copy)]
struct Scale(NonZeroUsize);

impl Scale {
    fn saturating_add(self, delta: i8) -> Self {
        let n = (self.0.get() as i8 + delta).max(1).min(100);
        Self(NonZeroUsize::new(n as usize).expect("unreachable"))
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self(NonZeroUsize::new(1).expect("unreachable"))
    }
}
