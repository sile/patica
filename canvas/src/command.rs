use pati::{ImageCommand, Point};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasCommand {
    Move(Point),
    Image(ImageCommand),
}
