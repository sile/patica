use crate::{log::Log, Color, ImageCommand, PatchEntry, PatchImageCommand, Point, Version};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    ops::{Bound, RangeBounds},
};

/// [`Image`] with a log of applied [`ImageCommand`]s.
#[derive(Debug, Default, Clone)]
pub struct VersionedImage {
    image: Image,
    log: Log,
}

impl VersionedImage {
    /// Makes a new [`VersionedImage`] instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the current version of this image.
    pub fn version(&self) -> Version {
        self.log.latest_image_version()
    }

    /// Gets the color of the pixel at the given point.
    pub fn get_pixel(&self, point: Point) -> Option<Color> {
        self.image.get_pixel(point)
    }

    /// Gets an iterator over the pixels in the given range.
    pub fn range_pixels<R>(&self, range: R) -> impl '_ + Iterator<Item = (Point, Color)>
    where
        R: RangeBounds<Point>,
    {
        self.image.range_pixels(range)
    }

    /// Gets the all pixels in this image.
    pub fn pixels(&self) -> &BTreeMap<Point, Color> {
        self.image.pixels()
    }

    /// Gets the all anchors in this image.
    pub fn anchors(&self) -> &BTreeMap<String, Point> {
        self.image.anchors()
    }

    /// Gets the all metadata in this image.
    pub fn metadata(&self) -> &BTreeMap<String, serde_json::Value> {
        self.image.metadata()
    }

    /// Applies the given command to this image.
    ///
    /// Returns `true` if the image is changed, otherwise `false`.
    /// If the command is applied, it is appended to the log.
    pub fn apply(&mut self, command: &ImageCommand) -> bool {
        let applied = self.image.apply(command);
        if applied {
            self.log
                .append_applied_command(command.clone(), &self.image);
        }
        applied
    }

    /// Gets the applied commands since the given version.
    pub fn applied_commands(&self, since: Version) -> &[ImageCommand] {
        let i = (since.0 as usize).min(self.log.commands().len());
        &self.log.commands()[i..]
    }

    /// Calculates the diff between the current image and the image at the given version.
    pub fn diff(&self, version: Version) -> Option<PatchImageCommand> {
        let image = self.log.restore_image(version)?;
        Some(self.image.diff(&image))
    }
}

/// Raster image.
#[derive(Debug, Default, Clone)]
pub struct Image {
    pixels: BTreeMap<Point, Color>,
    anchors: BTreeMap<String, Point>,
    metadata: BTreeMap<String, serde_json::Value>,
}

impl Image {
    /// Makes a new [`Image`] instance.
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

    /// Gets the all pixels in this image.
    pub fn pixels(&self) -> &BTreeMap<Point, Color> {
        &self.pixels
    }

    /// Gets the all anchors in this image.
    pub fn anchors(&self) -> &BTreeMap<String, Point> {
        &self.anchors
    }

    /// Gets the all metadata in this image.
    pub fn metadata(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.metadata
    }

    /// Applies the given command to this image.
    ///
    /// Returns `true` if the image is changed, otherwise `false`.
    pub fn apply(&mut self, command: &ImageCommand) -> bool {
        match command {
            ImageCommand::Patch(c) => self.handle_patch_command(c),
            ImageCommand::Anchor { name, point } => {
                if let Some(point) = *point {
                    self.anchors.insert(name.clone(), point) != Some(point)
                } else {
                    self.anchors.remove(name).is_some()
                }
            }
            ImageCommand::Put { name, value } => {
                if value.is_null() {
                    self.metadata.remove(name).is_some()
                } else {
                    self.metadata.insert(name.clone(), value.clone()) != Some(value.clone())
                }
            }
        }
    }

    fn handle_patch_command(&mut self, command: &PatchImageCommand) -> bool {
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

    fn diff(&self, other: &Self) -> PatchImageCommand {
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
        PatchImageCommand::new(entries)
    }
}

#[derive(Debug)]
struct RangePixels<'a> {
    image: &'a Image,
    start: Point,
    end: Point,
    row: std::collections::btree_map::Range<'a, Point, Color>,
}

impl<'a> RangePixels<'a> {
    fn new<R>(image: &'a Image, range: R) -> Self
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
        let row = image.pixels.range(start..=end);
        Self {
            image,
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
            self.row = self.image.pixels.range(self.start..=self.end);
        }
    }
}
