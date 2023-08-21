use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Color {
    Rgb(Rgb),
    Rgba(Rgba),
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgb(Rgb::new(r, g, b))
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::Rgba(Rgba::new(r, g, b, a))
    }

    pub fn to_rgba(self) -> Rgba {
        match self {
            Self::Rgb(rgb) => Rgba::new(rgb.r, rgb.g, rgb.b, 255),
            Self::Rgba(rgba) => rgba,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "(u8,u8,u8)", from = "(u8,u8,u8)")]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<(u8, u8, u8)> for Rgb {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}

impl From<Rgb> for (u8, u8, u8) {
    fn from(rgb: Rgb) -> Self {
        (rgb.r, rgb.g, rgb.b)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "(u8,u8,u8,u8)", from = "(u8,u8,u8,u8)")]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl From<(u8, u8, u8, u8)> for Rgba {
    fn from((r, g, b, a): (u8, u8, u8, u8)) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Rgba> for (u8, u8, u8, u8) {
    fn from(rgba: Rgba) -> Self {
        (rgba.r, rgba.g, rgba.b, rgba.a)
    }
}
