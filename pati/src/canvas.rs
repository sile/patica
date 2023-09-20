use crate::{log::Log, Color, Command, PatchCommand, PatchEntry, Point, Version};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    ops::{Bound, RangeBounds},
};

/// [`Canvas`] with a log of applied [`Command`]s.
#[derive(Debug, Default, Clone)]
pub struct VersionedCanvas {
    canvas: Canvas,
    log: Log,
}

impl VersionedCanvas {
    /// Makes a new [`VersionedCanvas`] instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the current version of this canvas.
    pub fn version(&self) -> Version {
        self.log.latest_canvas_version()
    }

    /// Gets the color of the pixel at the given point.
    pub fn get_pixel(&self, point: Point) -> Option<Color> {
        self.canvas.get_pixel(point)
    }

    /// Gets an iterator over the pixels in the given range.
    pub fn range_pixels<R>(&self, range: R) -> impl '_ + Iterator<Item = (Point, Color)>
    where
        R: RangeBounds<Point>,
    {
        self.canvas.range_pixels(range)
    }

    /// Gets the all pixels in this canvas.
    pub fn pixels(&self) -> &BTreeMap<Point, Color> {
        self.canvas.pixels()
    }

    /// Gets the all anchors in this canvas.
    pub fn anchors(&self) -> &BTreeMap<String, Point> {
        self.canvas.anchors()
    }

    /// Gets the all metadata in this canvas.
    pub fn metadata(&self) -> &BTreeMap<String, serde_json::Value> {
        self.canvas.metadata()
    }

    /// Applies the given command to this canvas.
    ///
    /// Returns `true` if the canvas is changed, otherwise `false`.
    /// If the command is applied, it is appended to the log.
    pub fn apply(&mut self, command: &Command) -> bool {
        let applied = self.canvas.apply(command);
        if applied {
            self.log
                .append_applied_command(command.clone(), &self.canvas);
        }
        applied
    }

    /// Gets the applied commands since the given version.
    pub fn applied_commands(&self, since: Version) -> &[Command] {
        let i = (since.0 as usize).min(self.log.commands().len());
        &self.log.commands()[i..]
    }

    /// Calculates the diff between the current canvas and the canvas at the given version.
    pub fn diff(&self, version: Version) -> Option<PatchCommand> {
        let canvas = self.log.restore_canvas(version)?;
        Some(self.canvas.diff(&canvas))
    }
}

/// Raster image canvas.
#[derive(Debug, Default, Clone)]
pub struct Canvas {
    pixels: BTreeMap<Point, Color>,
    anchors: BTreeMap<String, Point>,
    metadata: BTreeMap<String, serde_json::Value>,
}

impl Canvas {
    /// Makes a new [`Canvas`] instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the color of the pixel at the given point.
    pub fn get_pixel(&self, point: Point) -> Option<Color> {
        self.pixels.get(&point).copied()
    }

    /// Gets an iterator over the pixels in the given range.
    pub fn range_pixels<R>(&self, range: R) -> impl '_ + Iterator<Item = (Point, Color)>
    where
        R: RangeBounds<Point>,
    {
        RangePixels::new(self, range)
    }

    /// Gets the all pixels in this canvas.
    pub fn pixels(&self) -> &BTreeMap<Point, Color> {
        &self.pixels
    }

    /// Gets the all anchors in this canvas.
    pub fn anchors(&self) -> &BTreeMap<String, Point> {
        &self.anchors
    }

    /// Gets the all metadata in this canvas.
    pub fn metadata(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.metadata
    }

    /// Applies the given command to this canvas.
    ///
    /// Returns `true` if the canvas is changed, otherwise `false`.
    pub fn apply(&mut self, command: &Command) -> bool {
        match command {
            Command::Patch(c) => self.handle_patch_command(c),
            Command::Anchor { name, point } => {
                if let Some(point) = *point {
                    self.anchors.insert(name.clone(), point) != Some(point)
                } else {
                    self.anchors.remove(name).is_some()
                }
            }
            Command::Put { name, value } => {
                if value.is_null() {
                    self.metadata.remove(name).is_some()
                } else {
                    self.metadata.insert(name.clone(), value.clone()) != Some(value.clone())
                }
            }
        }
    }

    fn handle_patch_command(&mut self, command: &PatchCommand) -> bool {
        let mut applied = false;
        for entry in command.entries() {
            for point in &entry.points {
                if let Some(color) = entry.color {
                    applied |= self.pixels.insert(*point, color) != Some(color);
                } else {
                    applied |= self.pixels.remove(point).is_some();
                }
            }
        }
        applied
    }

    fn diff(&self, other: &Self) -> PatchCommand {
        let mut old_pixels = self.pixels.iter().map(|(p, c)| (*p, *c));
        let mut new_pixels = other.pixels.iter().map(|(p, c)| (*p, *c));

        let mut added: BTreeMap<Color, Vec<Point>> = BTreeMap::new();
        let mut removed: Vec<Point> = Vec::new();

        let mut old_pixel = old_pixels.next();
        let mut new_pixel = new_pixels.next();
        loop {
            match (old_pixel, new_pixel) {
                (None, None) => {
                    break;
                }
                (Some((point, _)), None) => {
                    removed.push(point);
                    old_pixel = old_pixels.next();
                }
                (None, Some((point, color))) => {
                    added.entry(color).or_default().push(point);
                    new_pixel = new_pixels.next();
                }
                (Some(old), Some(new)) => match old.0.cmp(&new.0) {
                    Ordering::Equal => {
                        if old.1 != new.1 {
                            added.entry(new.1).or_default().push(new.0);
                        }
                        old_pixel = old_pixels.next();
                        new_pixel = new_pixels.next();
                    }
                    Ordering::Less => {
                        removed.push(old.0);
                        old_pixel = old_pixels.next();
                    }
                    Ordering::Greater => {
                        added.entry(new.1).or_default().push(new.0);
                        new_pixel = new_pixels.next();
                    }
                },
            }
        }

        let mut entries = Vec::new();
        if !removed.is_empty() {
            entries.push(PatchEntry {
                color: None,
                points: removed,
            });
        }
        for (color, points) in added {
            entries.push(PatchEntry {
                color: Some(color),
                points,
            });
        }
        PatchCommand::new(entries)
    }
}

#[derive(Debug)]
struct RangePixels<'a> {
    canvas: &'a Canvas,
    start: Point,
    end: Point,
    row: std::collections::btree_map::Range<'a, Point, Color>,
}

impl<'a> RangePixels<'a> {
    fn new<R>(canvas: &'a Canvas, range: R) -> Self
    where
        R: RangeBounds<Point>,
    {
        let start = match range.start_bound() {
            Bound::Included(&p) => p,
            Bound::Excluded(&p) => Point::new(p.x + 1, p.y + 1),
            Bound::Unbounded => Point::new(i16::MIN, i16::MIN),
        };
        let end = match range.end_bound() {
            Bound::Included(&p) => p,
            Bound::Excluded(&p) => Point::new(p.x - 1, p.y - 1),
            Bound::Unbounded => Point::new(i16::MAX, i16::MAX),
        };
        let row = canvas.pixels.range(start..=end);
        Self {
            canvas,
            start,
            end,
            row,
        }
    }
}

impl<'a> Iterator for RangePixels<'a> {
    type Item = (Point, Color);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (point, color) = self.row.next()?;
            if self.start.y != point.y {
                self.start.y = point.y;
            } else if self.end.x < point.x {
                self.start.y += 1;
            } else {
                return Some((*point, *color));
            }
            self.row = self.canvas.pixels.range(self.start..=self.end);
        }
    }
}
