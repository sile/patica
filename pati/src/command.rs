use crate::{Color, Point};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{BufRead, Write},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Patch(PatchCommand),
    Anchor {
        name: String,
        point: Option<Point>,
    },
    Put {
        name: String,
        value: serde_json::Value,
    },
}

impl Command {
    pub fn patch(entries: Vec<PatchEntry>) -> Self {
        Self::Patch(PatchCommand::new(entries))
    }

    pub fn draw_pixels(pixels: impl Iterator<Item = (Point, Color)>) -> Self {
        let mut entries = BTreeMap::new();
        for (point, color) in pixels {
            entries
                .entry(color)
                .or_insert_with(|| PatchEntry {
                    color: Some(color),
                    points: Vec::new(),
                })
                .points
                .push(point);
        }
        Self::patch(entries.into_values().collect())
    }

    pub fn anchor(name: String, point: Option<Point>) -> Self {
        Self::Anchor { name, point }
    }

    pub fn set_anchor(name: &str, point: Point) -> Self {
        Self::Anchor {
            name: name.to_string(),
            point: Some(point),
        }
    }

    pub fn put(name: String, value: serde_json::Value) -> Self {
        Self::Put { name, value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchCommand(Vec<PatchEntry>);

impl PatchCommand {
    pub const fn new(entries: Vec<PatchEntry>) -> Self {
        Self(entries)
    }

    pub fn entries(&self) -> &[PatchEntry] {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchEntry {
    pub color: Option<Color>,
    pub points: Vec<Point>,
}

impl PatchEntry {
    pub fn color(color: Color, points: Vec<Point>) -> Self {
        Self {
            color: Some(color),
            points,
        }
    }

    pub fn erase(points: Vec<Point>) -> Self {
        Self {
            color: None,
            points,
        }
    }
}

#[derive(Debug)]
pub struct CommandWriter<W> {
    inner: W,
}

impl<W: Write> CommandWriter<W> {
    pub const fn new(inner: W) -> Self {
        Self { inner }
    }

    pub fn write_command(&mut self, command: &Command) -> std::io::Result<()> {
        serde_json::to_writer(&mut self.inner, command)?;
        writeln!(self.inner)?;
        self.inner.flush()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CommandReader<R> {
    inner: R,
    line: String,
}

impl<R: BufRead> CommandReader<R> {
    pub const fn new(inner: R) -> Self {
        Self {
            inner,
            line: String::new(),
        }
    }

    pub fn read_command(&mut self) -> std::io::Result<Option<Command>> {
        if 0 == self.inner.read_line(&mut self.line)? {
            Ok(None)
        } else if self.line.ends_with('\n') {
            let command = serde_json::from_str(&self.line)?;
            self.line.clear();
            Ok(Some(command))
        } else {
            Ok(None)
        }
    }
}
