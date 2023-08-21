use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "(i16, i16)", into = "(i16, i16)")]
pub struct CanvasPosition {
    pub x: i16,
    pub y: i16,
}

impl CanvasPosition {
    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

impl PartialOrd for CanvasPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.y, self.x).cmp(&(other.y, other.x)))
    }
}

impl Ord for CanvasPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.y, self.x).cmp(&(other.y, other.x))
    }
}

impl From<(i16, i16)> for CanvasPosition {
    fn from((x, y): (i16, i16)) -> Self {
        Self { x, y }
    }
}

impl Into<(i16, i16)> for CanvasPosition {
    fn into(self) -> (i16, i16) {
        (self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasRegion {
    pub top_left: CanvasPosition,
    pub bottom_right: CanvasPosition,
}
