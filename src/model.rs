use pagurus::image::Color;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Model {
    palette: Palette,
    pixels: BTreeMap<PixelPosition, ColorIndex>,
}

impl Model {
    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    pub fn handle_command(&mut self, command: &ModelCommand) -> pagurus::Result<()> {
        match command {
            _ => todo!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ModelCommand {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PixelPosition {
    pub y: u16,
    pub x: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColorIndex {
    pub y: u8,
    pub x: u8,
}

impl ColorIndex {
    pub const fn from_yx(y: u8, x: u8) -> Self {
        Self { y, x }
    }
}

#[derive(Debug)]
pub struct Palette {
    pub colors: BTreeMap<ColorIndex, Color>,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: [
                (ColorIndex::from_yx(0, 0), Color::rgb(255, 255, 255)),
                (ColorIndex::from_yx(0, 1), Color::rgb(255, 0, 0)),
                (ColorIndex::from_yx(0, 2), Color::rgb(0, 255, 0)),
                (ColorIndex::from_yx(0, 3), Color::rgb(0, 0, 255)),
                (ColorIndex::from_yx(0, 4), Color::rgb(0, 0, 0)),
            ]
            .into_iter()
            .collect(),
        }
    }
}
