use crate::{Color, Point};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{BufRead, Write},
};

/// [`Image`][crate::Image] command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageCommand {
    /// Patch command.
    Patch(PatchImageCommand),

    /// Anchor command.
    Anchor {
        /// Anchor name.
        name: String,

        /// Anchor point.
        point: Option<Point>,
    },

    /// Put command.
    Put {
        /// Metadata item name.
        name: String,

        /// Metadata item value.
        value: serde_json::Value,
    },
}

impl ImageCommand {
    /// Make a patch command from the given patch entries.
    pub const fn patch(entries: Vec<PatchEntry>) -> Self {
        Self::Patch(PatchImageCommand::new(entries))
    }

    /// Makes a patch command to draw the given pi xels.
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

    /// Makes an anchor command.
    pub fn anchor(name: impl Into<String>, point: Option<Point>) -> Self {
        Self::Anchor {
            name: name.into(),
            point,
        }
    }

    /// Makes a put command.
    pub fn put(name: impl Into<String>, value: serde_json::Value) -> Self {
        Self::Put {
            name: name.into(),
            value,
        }
    }
}

/// Patch command that is used to draw or erase pixels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchImageCommand(Vec<PatchEntry>);

impl PatchImageCommand {
    /// Makes a new [`PatchImageCommand`] instance.
    pub const fn new(entries: Vec<PatchEntry>) -> Self {
        Self(entries)
    }

    /// Gets the patch entries.
    pub fn entries(&self) -> &[PatchEntry] {
        &self.0
    }
}

/// Patch entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchEntry {
    /// Pixel color.
    ///
    /// If `None`, the pixels are erased.
    pub color: Option<Color>,

    /// Pixel points.
    pub points: Vec<Point>,
}

impl PatchEntry {
    /// Makes a new [`PatchEntry`] instance to draw pixels.
    pub const fn draw(color: Color, points: Vec<Point>) -> Self {
        Self {
            color: Some(color),
            points,
        }
    }

    /// Makes a new [`PatchEntry`] instance to erase pixels.
    pub const fn erase(points: Vec<Point>) -> Self {
        Self {
            color: None,
            points,
        }
    }
}

/// [`ImageCommand`] writer.
#[derive(Debug)]
pub struct ImageCommandWriter<W> {
    inner: W,
}

impl<W: Write> ImageCommandWriter<W> {
    /// Makes a new [`ImageCommandWriter`] instance.
    pub const fn new(inner: W) -> Self {
        Self { inner }
    }

    /// Writes the given command.
    pub fn write_command(&mut self, command: &ImageCommand) -> std::io::Result<()> {
        serde_json::to_writer(&mut self.inner, command)?;
        writeln!(self.inner)?;
        self.inner.flush()?;
        Ok(())
    }
}

/// [`ImageCommand`] reader.
#[derive(Debug)]
pub struct ImageCommandReader<R> {
    inner: R,
    line: String,
}

impl<R: BufRead> ImageCommandReader<R> {
    /// Makes a new [`ImageCommandReader`] instance.
    pub const fn new(inner: R) -> Self {
        Self {
            inner,
            line: String::new(),
        }
    }

    /// Reads a command.
    pub fn read_command(&mut self) -> std::io::Result<Option<ImageCommand>> {
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
