use pati::{Color, Point};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Editor {
    pixels: BTreeMap<Point, Color>,
}

impl Editor {
    pub fn new(pixels: BTreeMap<Point, Color>) -> Self {
        Self { pixels }
    }

    pub fn pixels(&self) -> impl '_ + Iterator<Item = (Point, Color)> {
        self.pixels.iter().map(|(p, c)| (*p, *c))
    }
}
