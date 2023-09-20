use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(into = "ColorLike", from = "ColorLike")]
pub struct Color {
    /// Red component.
    pub r: u8,

    /// Green component.
    pub g: u8,

    /// Blue component.
    pub b: u8,

    /// Alpha component.
    pub a: u8,
}

impl Color {
    /// Makes a [`Color`] instance with the given red, green, blue, and alpha 255.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Makes a [`Color`] instance with the given red, green, blue, and alpha.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::rgb(0, 0, 0)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
enum ColorLike {
    Rgb([u8; 3]),
    Rgba([u8; 4]),
}

impl From<Color> for ColorLike {
    fn from(color: Color) -> Self {
        match color.a {
            255 => Self::Rgb([color.r, color.g, color.b]),
            _ => Self::Rgba([color.r, color.g, color.b, color.a]),
        }
    }
}

impl From<ColorLike> for Color {
    fn from(color: ColorLike) -> Self {
        match color {
            ColorLike::Rgb([r, g, b]) => Self::rgb(r, g, b),
            ColorLike::Rgba([r, g, b, a]) => Self::rgba(r, g, b, a),
        }
    }
}

/// A point in 2D space.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "(i16, i16)", into = "(i16, i16)")]
pub struct Point {
    /// X coordinate.
    pub x: i16,

    /// Y coordinate.
    pub y: i16,
}

impl Point {
    /// The origin of [`Point`].
    pub const ORIGIN: Self = Self::new(0, 0);

    /// The minimum value of [`Point`].
    pub const MIN: Self = Self::new(i16::MIN, i16::MIN);

    /// The maximum value of [`Point`].
    pub const MAX: Self = Self::new(i16::MAX, i16::MAX);

    /// Makes a [`Point`] instance with the given x and y.
    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x.saturating_add(rhs.x), self.y.saturating_add(rhs.y))
    }
}

impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x.saturating_sub(rhs.x), self.y.saturating_sub(rhs.y))
    }
}

impl std::ops::Mul<i16> for Point {
    type Output = Self;

    fn mul(self, rhs: i16) -> Self::Output {
        Self::new(self.x.saturating_mul(rhs), self.y.saturating_mul(rhs))
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.y, self.x).cmp(&(other.y, other.x)))
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.y, self.x).cmp(&(other.y, other.x))
    }
}

impl From<(i16, i16)> for Point {
    fn from((x, y): (i16, i16)) -> Self {
        Self { x, y }
    }
}

impl From<Point> for (i16, i16) {
    fn from(point: Point) -> Self {
        (point.x, point.y)
    }
}
