use pati::{ImageCommand, Point};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasCommand {
    Move(Point),
    Scale(i8),
    Quit,
    Image(ImageCommand),
}
