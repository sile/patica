use crate::marker::MarkKind;
use pati::Point;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Move(MoveDestination),
    Mark(MarkKind),
    Pick,
    Cut,
    Cancel,
    Erase,
    Color,
    Paste,
    Undo,
    Redo,
    Quit,
    // Checkout
    // "o": [{"set": {"camera": [0, 0]}}],
    // "O": [{"set": {"camera": "origin"}}],
    // "+": [{"scale": 1}],
    // "-": [{"scale": -1}],
    // " ": {"if": {
    //     "neutral": [{"mark": "stroke"}, "color"],
    //     "marking": ["color"],
    //     "editing": ["paste", "cancel"]
    // }},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MoveDestination {
    Delta(Point),
    Anchor(String),
}
