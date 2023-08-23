use crate::{Color, Point, Version};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Patch(PatchCommand),
    Tag {
        name: String,
        version: Option<Version>,
    },
    Anchor {
        name: String,
        point: Option<Point>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchCommand(Vec<PatchEntry>);

impl PatchCommand {
    pub const fn new(entries: Vec<PatchEntry>) -> Self {
        Self(entries)
    }

    pub fn entries(&self) -> &[PatchEntry] {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchEntry {
    pub color: Option<Color>,
    pub points: Vec<Point>,
}
