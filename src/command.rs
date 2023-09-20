use crate::{clock::Ticks, frame::Frame, marker::MarkKind, model::Model, query::Query};
use orfail::OrFail;
use pati::{Color, Point};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU8;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Move(MoveDestination),
    Mark(MarkKind),
    Dip(Color),
    Pick,
    Cut,
    Copy,
    Cancel,
    Erase,
    Draw,
    Undo,
    Redo,
    Quit,
    Scale(i8),
    Center(CenterPoint),
    Anchor(String),
    BackgroundColor(Color),
    Import(Vec<(Point, Color)>),
    Embed(Frame),
    Tick(i32),
    Play(PlayCommand),
    Remove(RemoveTarget),
    Color(Color),
    Flip(FlipDirection),
    Rotate,
    ExternalCommand(ExternalCommand),
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
pub struct PlayCommand {
    #[serde(default)]
    pub offset: Ticks,
    pub duration: Ticks,
    pub fps: NonZeroU8,
    #[serde(default)]
    pub repeat: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoveTarget {
    Anchor(String),
    Frame(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlipDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalCommand {
    pub program: String,

    #[serde(default)]
    pub args: Vec<ExternalValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExternalValue {
    Static(String),
    Dynamic(ExternalValueDynamic),
    Concat(Vec<ExternalValue>),
}

impl ExternalValue {
    pub fn try_to_string(&self, model: &Model) -> orfail::Result<String> {
        match self {
            ExternalValue::Static(s) => Ok(s.clone()),
            ExternalValue::Dynamic(ExternalValueDynamic::Query(q)) => {
                let value = match model.query(q).unwrap_or(serde_json::Value::Null) {
                    serde_json::Value::Null => "".to_owned(),
                    serde_json::Value::Bool(v) => v.to_string(),
                    serde_json::Value::Number(v) => v.to_string(),
                    serde_json::Value::String(v) => v,
                    serde_json::Value::Array(_) => "".to_owned(), // TODO
                    serde_json::Value::Object(_) => "".to_owned(), // TODO
                };
                Ok(value)
            }
            ExternalValue::Dynamic(ExternalValueDynamic::QueryJson(q)) => {
                let value = serde_json::to_string(&model.query(q)).or_fail()?;
                Ok(value)
            }
            ExternalValue::Dynamic(ExternalValueDynamic::Env(name)) => {
                std::env::var(name).or_fail()
            }
            ExternalValue::Concat(list) => list
                .iter()
                .map(|v| v.try_to_string(model))
                .collect::<orfail::Result<Vec<_>>>()
                .map(|list| list.join("")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalValueDynamic {
    Query(Query),
    QueryJson(Query),
    Env(String),
}
