use crate::model::{Command, Model, PixelPosition, PixelRegion};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkKind {
    Line,
    Stroke,
    Fill,
    Rectangle,
    //  SameColor, InnerEdge, OuterEdge,
    // Rectangle, Ellipse
}

#[derive(Debug, Clone)]
pub enum Marker {
    Line(LineMarker),
    Stroke(StrokeMarker),
    Fill(FillMarker),
    Rectangle(RectangleMarker),
}

impl Marker {
    pub fn new(mark_kind: MarkKind, model: &Model) -> Self {
        match mark_kind {
            MarkKind::Line => Self::Line(LineMarker::new(model)),
            MarkKind::Stroke => Self::Stroke(StrokeMarker::new(model)),
            MarkKind::Fill => Self::Fill(FillMarker::new(model)),
            MarkKind::Rectangle => Self::Rectangle(RectangleMarker::new(model)),
        }
    }

    pub fn handle_command(&mut self, command: &Command, model: &Model) {
        match self {
            Self::Line(m) => m.handle_command(command, model),
            Self::Stroke(m) => m.handle_command(command, model),
            Self::Fill(m) => m.handle_command(command, model),
            Self::Rectangle(m) => m.handle_command(command, model),
        }
    }

    pub fn marked_pixels(&self) -> Box<dyn '_ + Iterator<Item = PixelPosition>> {
        match self {
            Self::Line(m) => Box::new(m.marked_pixels()),
            Self::Stroke(m) => Box::new(m.marked_pixels()),
            Self::Fill(m) => Box::new(m.marked_pixels()),
            Self::Rectangle(m) => Box::new(m.marked_pixels()),
        }
    }
}

fn line(start: PixelPosition, end: PixelPosition) -> impl Iterator<Item = PixelPosition> {
    LineMarker { start, end }.marked_pixels()
}

#[derive(Debug, Clone, Copy)]
pub struct LineMarker {
    start: PixelPosition,
    end: PixelPosition,
}

impl LineMarker {
    fn new(model: &Model) -> Self {
        Self {
            start: model.cursor().position(),
            end: model.cursor().position(),
        }
    }

    fn handle_command(&mut self, _command: &Command, model: &Model) {
        self.end = model.cursor().position();
    }

    fn marked_pixels(self) -> impl Iterator<Item = PixelPosition> {
        let p0 = self.start;
        let p1 = self.end;
        let dx = (p1.x - p0.x).abs() + 1;
        let dy = (p1.y - p0.y).abs() + 1;
        let sign_y = if p1.y > p0.y { 1 } else { -1 };
        let sign_x = if p1.x > p0.x { 1 } else { -1 };
        let (f, r, n, v0, sign0, mut v1, sign1) = if dx > dy {
            let f = xy as fn(i16, i16) -> PixelPosition;
            let r = Rational::new(dx, dy);
            (f, r, dx, p0.x, sign_x, p0.y, sign_y)
        } else {
            let f = yx as fn(i16, i16) -> PixelPosition;
            let r = Rational::new(dy, dx);
            (f, r, dy, p0.y, sign_y, p0.x, sign_x)
        };
        (0..n).map(move |i| {
            if i != 0 && (i - 1) / r != i / r {
                v1 += sign1;
            }
            f(v0 + i * sign0, v1)
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct Rational {
    num: i16,
    den: i16,
}

impl Rational {
    const fn new(num: i16, den: i16) -> Self {
        Self { num, den }
    }
}

impl std::ops::Div<Rational> for i16 {
    type Output = i16;

    fn div(self, rhs: Rational) -> Self::Output {
        self * rhs.den / rhs.num
    }
}

fn xy(x: i16, y: i16) -> PixelPosition {
    PixelPosition::from_xy(x, y)
}

fn yx(y: i16, x: i16) -> PixelPosition {
    PixelPosition::from_xy(x, y)
}

#[derive(Debug, Clone)]
pub struct StrokeMarker {
    stroke: HashSet<PixelPosition>,
    last: PixelPosition,
}

impl StrokeMarker {
    fn new(model: &Model) -> Self {
        Self {
            stroke: [model.cursor().position()].into_iter().collect(),
            last: model.cursor().position(),
        }
    }

    fn handle_command(&mut self, _command: &Command, model: &Model) {
        if self.last != model.cursor().position() {
            self.stroke
                .extend(line(self.last, model.cursor().position()));
            self.last = model.cursor().position();
        }
    }

    fn marked_pixels(&self) -> impl '_ + Iterator<Item = PixelPosition> {
        self.stroke.iter().copied()
    }
}

#[derive(Debug, Clone)]
pub struct FillMarker {
    cursor: PixelPosition,
    pixels: HashSet<PixelPosition>,
    region: PixelRegion,
    to_be_filled: bool,
}

impl FillMarker {
    fn new(model: &Model) -> Self {
        let mut this = Self {
            cursor: model.cursor().position(),
            pixels: HashSet::new(),
            region: model.pixels_region(),
            to_be_filled: false,
        };
        this.calc_pixels_to_be_filled(model);
        this
    }

    fn handle_command(&mut self, _command: &Command, model: &Model) {
        self.cursor = model.cursor().position();
        if !self.pixels.contains(&model.cursor().position()) {
            self.calc_pixels_to_be_filled(model);
        }
    }

    fn marked_pixels(&self) -> impl '_ + Iterator<Item = PixelPosition> {
        self.to_be_filled
            .then(|| self.pixels.iter().copied())
            .into_iter()
            .flatten()
    }

    fn calc_pixels_to_be_filled(&mut self, model: &Model) {
        self.pixels.clear();
        self.to_be_filled = true;

        let color = model.get_pixel_color(self.cursor);
        let mut stack = vec![self.cursor];
        while let Some(p) = stack.pop() {
            if self.pixels.contains(&p) {
                continue;
            }
            if model.get_pixel_color(p) != color {
                continue;
            }
            if !self.region.contains(p) {
                self.to_be_filled = false;
                break;
            }

            self.pixels.insert(p);
            stack.push(PixelPosition::from_xy(p.x - 1, p.y));
            stack.push(PixelPosition::from_xy(p.x + 1, p.y));
            stack.push(PixelPosition::from_xy(p.x, p.y - 1));
            stack.push(PixelPosition::from_xy(p.x, p.y + 1));
        }
    }
}

#[derive(Debug, Clone)]
pub struct RectangleMarker {
    start: PixelPosition,
    end: PixelPosition,
}

impl RectangleMarker {
    fn new(model: &Model) -> Self {
        Self {
            start: model.cursor().position(),
            end: model.cursor().position(),
        }
    }

    fn handle_command(&mut self, _command: &Command, model: &Model) {
        self.end = model.cursor().position();
    }

    fn marked_pixels(&self) -> impl Iterator<Item = PixelPosition> {
        let min_x = self.start.x.min(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_x = self.start.x.max(self.end.x);
        let max_y = self.start.y.max(self.end.y);
        PixelRegion::from_corners(min_x, min_y, max_x, max_y).edge_pixels()
    }
}
