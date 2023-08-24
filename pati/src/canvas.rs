use crate::{log::Log, Color, Command, PatchCommand, PatchEntry, Point, Version};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    ops::{Bound, RangeBounds},
};

#[derive(Debug, Default, Clone)]
pub struct VersionedCanvas {
    canvas: Canvas,
    log: Log,
}

impl VersionedCanvas {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn version(&self) -> Version {
        self.log.latest_canvas_version()
    }

    pub fn get_pixel(&self, point: Point) -> Option<Color> {
        self.canvas.get_pixel(point)
    }

    pub fn range_pixels<R>(&self, range: R) -> impl '_ + Iterator<Item = (Point, Color)>
    where
        R: RangeBounds<Point>,
    {
        self.canvas.range_pixels(range)
    }

    pub fn pixels(&self) -> &BTreeMap<Point, Color> {
        self.canvas.pixels()
    }

    pub fn tags(&self) -> &BTreeMap<String, Version> {
        self.canvas.tags()
    }

    pub fn anchors(&self) -> &BTreeMap<String, Point> {
        self.canvas.anchors()
    }

    pub fn apply(&mut self, command: &Command) -> bool {
        let applied = self.canvas.apply(command);
        if applied {
            self.log
                .append_applied_command(command.clone(), &self.canvas);
        }
        applied
    }

    pub fn applied_commands(&self, since: Version) -> &[Command] {
        let i = (since.0 as usize).min(self.log.commands().len());
        &self.log.commands()[i..]
    }

    pub fn diff(&self, version: Version) -> Option<PatchCommand> {
        let canvas = self.log.restore_canvas(version)?;
        Some(canvas.diff(&canvas))
    }
}

#[derive(Debug, Default, Clone)]
pub struct Canvas {
    pixels: BTreeMap<Point, Color>,
    tags: BTreeMap<String, Version>,
    anchors: BTreeMap<String, Point>,
}

impl Canvas {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_pixel(&self, point: Point) -> Option<Color> {
        self.pixels.get(&point).copied()
    }

    pub fn range_pixels<R>(&self, range: R) -> impl '_ + Iterator<Item = (Point, Color)>
    where
        R: RangeBounds<Point>,
    {
        RangePixels::new(self, range)
    }

    pub fn pixels(&self) -> &BTreeMap<Point, Color> {
        &self.pixels
    }

    pub fn tags(&self) -> &BTreeMap<String, Version> {
        &self.tags
    }

    pub fn anchors(&self) -> &BTreeMap<String, Point> {
        &self.anchors
    }

    pub fn apply(&mut self, command: &Command) -> bool {
        match command {
            Command::Patch(c) => self.handle_patch_command(c),
            Command::Tag { name, version } => {
                if let Some(version) = *version {
                    self.tags.insert(name.clone(), version) != Some(version)
                } else {
                    self.tags.remove(name).is_some()
                }
            }
            Command::Anchor { name, point } => {
                if let Some(point) = *point {
                    self.anchors.insert(name.clone(), point) != Some(point)
                } else {
                    self.anchors.remove(name).is_some()
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
                            old_pixel = old_pixels.next();
                            new_pixel = new_pixels.next();
                        }
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

        let row = canvas
            .pixels
            .range(start..=Point::new(start.y.min(end.y), end.x));
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
            if let Some((point, color)) = self.row.next() {
                return Some((*point, *color));
            }

            if self.start.y >= self.end.y {
                return None;
            }

            self.start.y += 1;
            self.row = self
                .canvas
                .pixels
                .range(self.start..=Point::new(self.start.y, self.end.x));
        }
    }
}
