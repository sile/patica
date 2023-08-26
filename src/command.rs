use std::num::NonZeroU8;

use crate::{clock::Ticks, frame::Frame, marker::MarkKind};
use pati::{Color, Point};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Move(MoveDestination),
    Mark(MarkKind),
    Dip(Color),
    Pick,
    Cut,
    Cancel,
    Erase,
    Draw,
    Undo,
    Redo,
    Quit,
    Scale(i8),
    Center(CenterPoint),
    Anchor(String),
    Tag(String),
    BackgroundColor(Color),
    Repeat(u8),
    Checkout(Checkout),
    Import(Vec<(Point, Color)>),
    Embed(Frame),
    Tick(i32),
    Play(PlayCommand),
    // Edit(rotate|flip|color)
    // Rotate
    // Flip
    // {"remove": {"tag"|"anchor"|"frame": "name"}}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MoveDestination {
    Delta(Point),
    Anchor(AnchorName),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorName {
    pub anchor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CenterPoint {
    Cursor,
    Anchor(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Checkout {
    Tag(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayCommand {
    #[serde(default)]
    pub offset: Ticks,
    pub duration: Ticks,
    pub fps: NonZeroU8,
}
