use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Put(PutCommand),
    Remove(RemoveCommand),
    // Dip(Color)
    // Pick
    // Color
    // Erase
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PutCommand {
    Frame,
    Metadata(BTreeMap<String, serde_json::Value>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoveCommand {
    Frame(String),
    Metadata(String),
}
