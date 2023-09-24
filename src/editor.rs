use crate::command::FlipDirection;
use pati::{Color, Point};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Editor {
    pixels: BTreeMap<Point, Color>,
}

impl Editor {
    pub fn new(pixels: BTreeMap<Point, Color>) -> Self {
        Self { pixels }
    }

    pub fn pixels(&self) -> impl '_ + Iterator<Item = (Point, Color)> {
        self.pixels.iter().map(|(p, c)| (*p, *c))
    }

    pub fn apply_color(&mut self, color: Color) {
        for c in self.pixels.values_mut() {
            *c = color;
        }
    }

    pub fn apply_rotate(&mut self) {
        let center = self.center();
        self.pixels = self
            .pixels
            .iter()
            .map(|(&p, &c)| {
                let x = center.x + (center.y - p.y);
                let y = center.y + (p.x - center.x);
                (Point::new(x, y), c)
            })
            .collect();
    }

    pub fn apply_flip(&mut self, direction: FlipDirection) {
        let center = self.center();
        match direction {
            FlipDirection::Horizontal => {
                self.pixels = self
                    .pixels
                    .iter()
                    .map(|(&p, &c)| {
                        let x = center.x + (center.x - p.x);
                        (Point::new(x, p.y), c)
                    })
                    .collect();
            }
            FlipDirection::Vertical => {
                self.pixels = self
                    .pixels
                    .iter()
                    .map(|(&p, &c)| {
                        let y = center.y + (center.y - p.y);
                        (Point::new(p.x, y), c)
                    })
                    .collect();
            }
        }
    }

    fn center(&self) -> Point {
        if self.pixels.is_empty() {
            return Point::new(0, 0);
        }

        let mut start = Point::MAX;
        let mut end = Point::MIN;

        for point in self.pixels.keys().copied() {
            start.x = start.x.min(point.x);
            start.y = start.y.min(point.y);
            end.x = end.x.max(point.x);
            end.y = end.y.max(point.y);
        }

        Point::new(
            (end.x - start.x) / 2 + start.x,
            (end.y - start.y) / 2 + start.y,
        )
    }
}
