use crate::{
    log::{FullLog, Log, NullLog},
    Color, Command, PatchCommand, Point, Version,
};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub struct VersionedCanvas(CanvasInner<FullLog>);

impl VersionedCanvas {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn version(&self) -> Version {
        self.0.version()
    }

    pub fn pixels(&self) -> &BTreeMap<Point, Color> {
        self.0.pixels()
    }

    pub fn tags(&self) -> &BTreeMap<String, Version> {
        self.0.tags()
    }

    pub fn anchors(&self) -> &BTreeMap<String, Point> {
        self.0.anchors()
    }

    pub fn apply(&mut self, command: Command) -> bool {
        self.0.apply(command)
    }

    pub fn diff(&self, version: Version) -> Option<PatchCommand> {
        self.0.diff(version)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Canvas(CanvasInner<NullLog>);

impl Canvas {
    pub fn new() -> Self {
        Self(Default::default())
    }

    pub fn pixels(&self) -> &BTreeMap<Point, Color> {
        self.0.pixels()
    }

    pub fn tags(&self) -> &BTreeMap<String, Version> {
        self.0.tags()
    }

    pub fn anchors(&self) -> &BTreeMap<String, Point> {
        self.0.anchors()
    }

    pub fn apply(&mut self, command: Command) -> bool {
        self.0.apply(command)
    }
}

#[derive(Debug, Default, Clone)]
pub struct CanvasInner<L> {
    state: CanvasState,
    log: L,
}

impl<L: Log> CanvasInner<L> {
    pub fn new() -> Self {
        Self {
            state: CanvasState::default(),
            log: L::default(),
        }
    }

    pub fn version(&self) -> Version {
        self.log.latest_state_version()
    }

    pub fn pixels(&self) -> &BTreeMap<Point, Color> {
        &self.state.pixels
    }

    pub fn tags(&self) -> &BTreeMap<String, Version> {
        &self.state.tags
    }

    pub fn anchors(&self) -> &BTreeMap<String, Point> {
        &self.state.anchors
    }

    pub fn apply(&mut self, command: Command) -> bool {
        let applied = self.state.apply(&command);
        if applied {
            self.log.append_applied_command(command, &self.state);
        }
        applied
    }

    pub fn diff(&self, _version: Version) -> Option<PatchCommand> {
        todo!()
    }
}

// TODO(?): private
#[derive(Debug, Default, Clone)]
pub struct CanvasState {
    pixels: BTreeMap<Point, Color>,
    tags: BTreeMap<String, Version>,
    anchors: BTreeMap<String, Point>,
}

impl CanvasState {
    pub fn apply(&mut self, command: &Command) -> bool {
        match command {
            Command::Patch(c) => self.handle_patch_command(c),
            Command::Tag { name, version } => {
                if let Some(version) = *version {
                    self.tags.insert(name.clone(), version) != Some(version)
                } else {
                    self.tags.remove(name).is_some()
                }
            }
            Command::Anchor { name, point } => {
                if let Some(point) = *point {
                    self.anchors.insert(name.clone(), point) != Some(point)
                } else {
                    self.anchors.remove(name).is_some()
                }
            }
        }
    }

    fn handle_patch_command(&mut self, command: &PatchCommand) -> bool {
        let mut applied = false;
        for entry in command.entries() {
            //
        }
        applied
    }
}
