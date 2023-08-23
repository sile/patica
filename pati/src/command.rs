use crate::{Color, Point, Version};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Patch(PatchCommand),
    Tag {
        name: String,
        version: Option<Version>,
    },
    Anchor {
        name: String,
        point: Option<Point>,
    },
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
        loop {
            if 0 == self.inner.read_line(&mut self.line)? {
                return Ok(None);
            }
            if self.line.ends_with('\n') {
                let command = serde_json::from_str(&self.line)?;
                self.line.clear();
                return Ok(Some(command));
            } else {
                return Ok(None);
            }
        }
    }
}
