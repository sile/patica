use crate::clock::Ticks;
use orfail::OrFail;
use pati::{Color, Point, Version, VersionedImage};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub name: String,
    pub path: PathBuf,
    pub top_left_anchor: String,
    pub bottom_right_anchor: String,
    pub start_ticks: Ticks,
    pub end_ticks: Ticks,
}

impl Frame {
    pub fn is_visible(&self, ticks: Ticks) -> bool {
        (self.start_ticks..self.end_ticks).contains(&ticks)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedFrame {
    pub frame: Frame,
    pub start: Point,
    pub version: Version,
    pub pixels: BTreeMap<Point, Color>,
}

impl EmbeddedFrame {
    pub fn new(frame: Frame, start: Point) -> Self {
        Self {
            frame,
            start,
            version: Version::default(),
            pixels: BTreeMap::new(),
        }
    }

    pub fn sync(&mut self, canvas: &VersionedImage) -> orfail::Result<()> {
        let start = canvas
            .anchors()
            .get(&self.frame.top_left_anchor)
            .copied()
            .or_fail()?;
        let end = canvas
            .anchors()
            .get(&self.frame.bottom_right_anchor)
            .copied()
            .or_fail()?;
        self.version = canvas.version();
        self.pixels = canvas
            .range_pixels(start..=end)
            .map(|(p, c)| ((p - start) + self.start, c))
            .collect();
        Ok(())
    }
}
