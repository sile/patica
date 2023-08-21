use crate::{
    color::{Color, Rgba},
    command::{Command, Metadata, PutCommand, RemoveCommand},
    spatial::{Point, RectangularArea},
};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Canvas {
    cursor: Point,
    brush_color: Color,
    pixels: Pixels,
    metadata: Metadata,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            cursor: Point::default(),
            brush_color: Color::rgb(0, 0, 0),
            pixels: Pixels::default(),
            metadata: Metadata::default(),
        }
    }

    pub fn cursor(&self) -> Point {
        self.cursor
    }

    pub fn brush_color(&self) -> Color {
        self.brush_color
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn pixels(&self) -> &Pixels {
        &self.pixels
    }

    pub fn drawing_area(&self) -> RectangularArea {
        RectangularArea::from_points(self.pixels.iter().map(|(point, _)| point))
    }

    pub fn apply(&mut self, command: Command) -> bool {
        let applied = match command {
            Command::Put(c) => self.handle_put_command(c),
            Command::Remove(c) => self.handle_remove_command(c),
        };
        applied
    }

    fn handle_put_command(&mut self, command: PutCommand) -> bool {
        match command {
            PutCommand::Metadata(m) => {
                if m.is_empty() {
                    return false;
                }
                for (name, value) in m.into_iter() {
                    self.metadata.put(name, value);
                }
            }
        }
        true
    }

    fn handle_remove_command(&mut self, command: RemoveCommand) -> bool {
        match command {
            RemoveCommand::Metadata(name) => self.metadata.remove(&name).is_some(),
        }
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Pixels(BTreeMap<Point, Rgba>);

impl Pixels {
    pub fn get(&self, point: Point) -> Option<Rgba> {
        self.0.get(&point).copied()
    }

    pub fn iter(&self) -> impl '_ + Iterator<Item = (Point, Rgba)> {
        self.0.iter().map(|(point, color)| (*point, *color))
    }

    pub fn area(&self, area: RectangularArea) -> impl '_ + Iterator<Item = (Point, Rgba)> {
        AreaPixels::new(self, area)
    }
}

#[derive(Debug)]
struct AreaPixels<'a> {
    pixels: &'a Pixels,
    area: RectangularArea,
    row_iter: Option<std::collections::btree_map::Range<'a, Point, Rgba>>,
}

impl<'a> AreaPixels<'a> {
    fn new(pixels: &'a Pixels, area: RectangularArea) -> Self {
        Self {
            pixels,
            area,
            row_iter: None,
        }
    }
}

impl<'a> Iterator for AreaPixels<'a> {
    type Item = (Point, Rgba);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((point, color)) = self.row_iter.as_mut().and_then(|i| i.next()) {
                return Some((*point, *color));
            }

            let range = self.area.next_row_range()?;
            self.row_iter = Some(self.pixels.0.range(range));
        }
    }
}
