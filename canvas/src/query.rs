use pati::{Color, Point};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU8;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasQuery {
    Cursor,
    Camera,
    BrushColor,
    BackgroundColor,
    Scale,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasQueryValue {
    Cursor(Point),
    Camera(Point),
    BrushColor(Color),
    BackgroundColor(Color),
    Scale(NonZeroU8),
}
