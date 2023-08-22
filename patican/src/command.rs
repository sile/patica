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
    //Frame,
    Metadata(Metadata),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoveCommand {
    //Frame(String),
    Metadata(String),
}

// TODO: move
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata(BTreeMap<String, serde_json::Value>);

impl Metadata {
    pub fn get(&self, name: &str) -> Option<&serde_json::Value> {
        self.0.get(name)
    }

    pub fn put(&mut self, name: String, value: serde_json::Value) {
        self.0.insert(name, value);
    }

    pub fn remove(&mut self, name: &str) -> Option<serde_json::Value> {
        self.0.remove(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &serde_json::Value)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
