use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, ops::RangeInclusive};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "(i16, i16)", into = "(i16, i16)")]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Point {
    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x.saturating_add(rhs.x), self.y.saturating_add(rhs.y))
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.y, self.x).cmp(&(other.y, other.x)))
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.y, self.x).cmp(&(other.y, other.x))
    }
}

impl From<(i16, i16)> for Point {
    fn from((x, y): (i16, i16)) -> Self {
        Self { x, y }
    }
}

impl From<Point> for (i16, i16) {
    fn from(point: Point) -> Self {
        (point.x, point.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectangularArea {
    pub top_left: Point,
    pub bottom_right: Point,
}

impl RectangularArea {
    pub fn from_points(points: impl Iterator<Item = Point>) -> Self {
        let mut top_left = Point::new(i16::MAX, i16::MAX);
        let mut bottom_right = Point::new(i16::MIN, i16::MIN);
        for point in points {
            top_left.x = top_left.x.min(point.x);
            top_left.y = top_left.y.min(point.y);
            bottom_right.x = bottom_right.x.max(point.x);
            bottom_right.y = bottom_right.y.max(point.y);
        }
        Self {
            top_left,
            bottom_right,
        }
    }

    pub fn is_empty(self) -> bool {
        self.top_left.x > self.bottom_right.x || self.top_left.y > self.bottom_right.y
    }

    pub fn width(self) -> u16 {
        (self.bottom_right.x - self.top_left.x + 1).max(0) as u16
    }

    pub fn height(self) -> u16 {
        (self.bottom_right.y - self.top_left.y + 1).max(0) as u16
    }

    pub fn next_row_range(&mut self) -> Option<RangeInclusive<Point>> {
        if self.is_empty() {
            return None;
        }
        let start = self.top_left;
        let end = Point::new(self.bottom_right.x, start.y);
        self.top_left.y += 1;
        Some(start..=end)
    }
}
