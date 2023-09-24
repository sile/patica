use pati::{Color, Point};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasQuery {
    Cursor,
    Camera,
    BrushColor,
    BackgroundColor,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasQueryValue {
    Cursor(Point),
    Camera(Point),
    BrushColor(Color),
    BackgroundColor(Color),
}
