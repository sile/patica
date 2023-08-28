use crate::model::Model;
use pati::{Color, Point};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkKind {
    Line,
    Stroke,
    Fill,
    Rectangle,
    Ellipse,
    Region,
    Color,
    All,
}

#[derive(Debug, Clone)]
pub enum Marker {
    Line(LineMarker),
    Stroke(StrokeMarker),
    Fill(FillMarker),
    Rectangle(RectangleMarker),
    Ellipse(EllipseMarker),
    Region(RegionMarker),
    Color(ColorMarker),
    All(AllMarker),
}

impl Marker {
    pub fn new(mark_kind: MarkKind, model: &Model) -> Self {
        match mark_kind {
            MarkKind::Line => Self::Line(LineMarker::new(model)),
            MarkKind::Stroke => Self::Stroke(StrokeMarker::new(model)),
            MarkKind::Fill => Self::Fill(FillMarker::new(model)),
            MarkKind::Rectangle => Self::Rectangle(RectangleMarker::new(model)),
            MarkKind::Region => Self::Region(RegionMarker::new(model)),
            MarkKind::Ellipse => Self::Ellipse(EllipseMarker::new(model)),
            MarkKind::Color => Self::Color(ColorMarker::new(model)),
            MarkKind::All => Self::All(AllMarker::new(model)),
        }
    }

    pub fn handle_move(&mut self, model: &Model) {
        match self {
            Self::Line(m) => m.handle_move(model),
            Self::Stroke(m) => m.handle_mvoe(model),
            Self::Fill(m) => m.handle_move(model),
            Self::Rectangle(m) => m.handle_move(model),
            Self::Region(m) => m.handle_move(model),
            Self::Ellipse(m) => m.handle_move(model),
            Self::Color(m) => m.handle_move(model),
            Self::All(m) => m.handle_move(model),
        }
    }

    pub fn marked_points(&self) -> Box<dyn '_ + Iterator<Item = Point>> {
        match self {
            Self::Line(m) => Box::new(m.marked_points()),
            Self::Stroke(m) => Box::new(m.marked_points()),
            Self::Fill(m) => Box::new(m.marked_points()),
            Self::Rectangle(m) => Box::new(m.marked_points()),
            Self::Region(m) => Box::new(m.marked_points()),
            Self::Ellipse(m) => Box::new(m.marked_points()),
            Self::Color(m) => Box::new(m.marked_points()),
            Self::All(m) => Box::new(m.marked_points()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LineMarker {
    start: Point,
    end: Point,
}

impl LineMarker {
    fn new(model: &Model) -> Self {
        Self {
            start: model.cursor(),
            end: model.cursor(),
        }
    }

    fn line(start: Point, end: Point) -> impl Iterator<Item = Point> {
        Self { start, end }.marked_points()
    }

    fn handle_move(&mut self, model: &Model) {
        self.end = model.cursor();
    }

    fn marked_points(self) -> impl Iterator<Item = Point> {
        let p0 = self.start;
        let p1 = self.end;
        let dx = (p1.x - p0.x).abs() + 1;
        let dy = (p1.y - p0.y).abs() + 1;
        let sign_y = if p1.y > p0.y { 1 } else { -1 };
        let sign_x = if p1.x > p0.x { 1 } else { -1 };
        let (f, r, n, v0, sign0, mut v1, sign1) = if dx > dy {
            let f = xy as fn(i16, i16) -> Point;
            let r = Rational::new(dx, dy);
            (f, r, dx, p0.x, sign_x, p0.y, sign_y)
        } else {
            let f = yx as fn(i16, i16) -> Point;
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

fn xy(x: i16, y: i16) -> Point {
    Point::new(x, y)
}

fn yx(y: i16, x: i16) -> Point {
    Point::new(x, y)
}

#[derive(Debug, Clone)]
pub struct StrokeMarker {
    stroke: HashSet<Point>,
    last: Point,
}

impl StrokeMarker {
    fn new(model: &Model) -> Self {
        Self {
            stroke: [model.cursor()].into_iter().collect(),
            last: model.cursor(),
        }
    }

    fn handle_mvoe(&mut self, model: &Model) {
        let cursor = model.cursor();
        if self.last != cursor {
            self.stroke.extend(LineMarker::line(self.last, cursor));
            self.last = cursor;
        }
    }

    fn marked_points(&self) -> impl '_ + Iterator<Item = Point> {
        self.stroke.iter().copied()
    }
}

#[derive(Debug, Clone, Copy)]
struct Region {
    top_left: Point,
    bottom_right: Point,
}

impl Region {
    fn from_points(points: impl Iterator<Item = Point>) -> Self {
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

    fn start(self) -> Point {
        self.top_left
    }

    fn end(self) -> Point {
        self.bottom_right
    }

    fn contains(self, point: Point) -> bool {
        self.top_left.x <= point.x
            && point.x <= self.bottom_right.x
            && self.top_left.y <= point.y
            && point.y <= self.bottom_right.y
    }

    fn points(self) -> impl Iterator<Item = Point> {
        let Point { x: x0, y: y0 } = self.top_left;
        let Point { x: x1, y: y1 } = self.bottom_right;
        (y0..=y1).flat_map(move |y| (x0..=x1).map(move |x| Point::new(x, y)))
    }

    fn edge_points(self) -> impl Iterator<Item = Point> {
        let Point { x: x0, y: y0 } = self.top_left;
        let Point { x: x1, y: y1 } = self.bottom_right;
        [
            Point::new(x0, y0),
            Point::new(x1, y0),
            Point::new(x0, y1),
            Point::new(x1, y1),
        ]
        .into_iter()
        .chain((x0 + 1..x1).map(move |x| Point::new(x, y0)))
        .chain((x0 + 1..x1).map(move |x| Point::new(x, y1)))
        .chain((y0 + 1..y1).map(move |y| Point::new(x0, y)))
        .chain((y0 + 1..y1).map(move |y| Point::new(x1, y)))
    }
}

#[derive(Debug, Clone)]
pub struct FillMarker {
    cursor: Point,
    points: HashSet<Point>,
    region: Region,
    to_be_filled: bool,
}

impl FillMarker {
    fn new(model: &Model) -> Self {
        let mut this = Self {
            cursor: model.cursor(),
            points: HashSet::new(),
            region: Region::from_points(
                std::iter::once(model.cursor()).chain(model.canvas().pixels().keys().copied()),
            ),
            to_be_filled: false,
        };
        this.calc_points_to_be_filled(model);
        this
    }

    fn handle_move(&mut self, model: &Model) {
        self.cursor = model.cursor();
        if !self.points.contains(&model.cursor()) {
            self.calc_points_to_be_filled(model);
        }
    }

    fn marked_points(&self) -> impl '_ + Iterator<Item = Point> {
        self.to_be_filled
            .then(|| self.points.iter().copied())
            .into_iter()
            .flatten()
    }

    fn calc_points_to_be_filled(&mut self, model: &Model) {
        self.points.clear();
        self.to_be_filled = true;

        let color = model.canvas().get_pixel(self.cursor);
        let mut stack = vec![self.cursor];
        while let Some(p) = stack.pop() {
            if self.points.contains(&p) {
                continue;
            }
            if model.canvas().get_pixel(p) != color {
                continue;
            }
            if !self.region.contains(p) {
                self.to_be_filled = false;
                break;
            }

            self.points.insert(p);
            stack.push(Point::new(p.x - 1, p.y));
            stack.push(Point::new(p.x + 1, p.y));
            stack.push(Point::new(p.x, p.y - 1));
            stack.push(Point::new(p.x, p.y + 1));
        }
    }
}

#[derive(Debug, Clone)]
pub struct RectangleMarker {
    start: Point,
    end: Point,
}

impl RectangleMarker {
    fn new(model: &Model) -> Self {
        Self {
            start: model.cursor(),
            end: model.cursor(),
        }
    }

    fn handle_move(&mut self, model: &Model) {
        self.end = model.cursor();
    }

    fn marked_points(&self) -> impl Iterator<Item = Point> {
        Region::from_points([self.start, self.end].into_iter()).edge_points()
    }
}

#[derive(Debug, Clone)]
pub struct RegionMarker {
    inner: RectangleMarker,
}

impl RegionMarker {
    fn new(model: &Model) -> Self {
        Self {
            inner: RectangleMarker::new(model),
        }
    }

    fn handle_move(&mut self, model: &Model) {
        self.inner.handle_move(model);
    }

    fn marked_points(&self) -> impl Iterator<Item = Point> {
        Region::from_points([self.inner.start, self.inner.end].into_iter()).points()
    }
}

#[derive(Debug, Clone)]
pub struct EllipseMarker {
    start: Point,
    cursor: Point,
    points: HashSet<Point>,
}

impl EllipseMarker {
    fn new(model: &Model) -> Self {
        Self {
            start: model.cursor(),
            cursor: model.cursor(),
            points: vec![model.cursor()].into_iter().collect(),
        }
    }

    fn handle_move(&mut self, model: &Model) {
        if self.cursor != model.cursor() {
            self.cursor = model.cursor();
            self.calc_points();
        }
    }

    fn marked_points(&self) -> impl '_ + Iterator<Item = Point> {
        self.points.iter().copied()
    }

    fn calc_points(&mut self) {
        self.points.clear();

        let region = Region::from_points([self.start, self.cursor].into_iter());

        let x_radius = (region.end().x as f32 - region.start().x as f32) / 2.0;
        let y_radius = (region.end().y as f32 - region.start().y as f32) / 2.0;
        let x_radius2 = x_radius.powi(2);
        let y_radius2 = y_radius.powi(2);
        let center_x = x_radius + region.start().x as f32;
        let center_y = y_radius + region.start().y as f32;

        let ratio = |xi: f32, yi: f32| {
            let mut count = 0;
            for xj in 0..=10 {
                for yj in 0..=10 {
                    let xv = (xi + 0.1 * xj as f32).powi(2) / x_radius2;
                    let yv = (yi + 0.1 * yj as f32).powi(2) / y_radius2;
                    if xv + yv <= 1.0 {
                        count += 1;
                    }
                }
            }
            count as f32 / (11 * 11) as f32
        };

        let mut xi = x_radius.fract();
        let mut yi = y_radius - 1.0;
        while xi < x_radius && yi >= 0.0 {
            let px = (center_x + xi) as i16;
            let mx = (center_x - xi) as i16;
            let py = (center_y + yi) as i16;
            let my = (center_y - yi) as i16;
            self.points.insert(Point::new(px, py));
            self.points.insert(Point::new(mx, my));
            self.points.insert(Point::new(px, my));
            self.points.insert(Point::new(mx, py));

            if ratio(xi + 1.0, yi) >= 0.5 {
                xi += 1.0;
            } else if ratio(xi + 1.0, yi - 1.0) >= 0.5 {
                xi += 1.0;
                yi -= 1.0;
            } else {
                yi -= 1.0;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorMarker {
    color: Option<Color>,
    points: HashSet<Point>,
}

impl ColorMarker {
    fn new(model: &Model) -> Self {
        let color = model.canvas().get_pixel(model.cursor());
        let mut this = Self {
            color,
            points: HashSet::new(),
        };
        this.calc_points(model);
        this
    }

    fn handle_move(&mut self, model: &Model) {
        let color = model.canvas().get_pixel(model.cursor());
        if self.color != color {
            self.color = color;
            self.calc_points(model);
        }
    }

    fn marked_points(&self) -> impl '_ + Iterator<Item = Point> {
        self.points.iter().copied()
    }

    fn calc_points(&mut self, model: &Model) {
        self.points.clear();
        let Some(color) = self.color else {
            return;
        };
        self.points = model
            .canvas()
            .pixels()
            .iter()
            .filter(|p| *p.1 == color)
            .map(|p| *p.0)
            .collect();
    }
}

#[derive(Debug, Clone)]
pub struct AllMarker {
    points: HashSet<Point>,
}

impl AllMarker {
    fn new(model: &Model) -> Self {
        Self {
            points: model.canvas().pixels().keys().copied().collect(),
        }
    }

    fn handle_move(&mut self, model: &Model) {
        self.points = model.canvas().pixels().keys().copied().collect();
    }

    fn marked_points(&self) -> impl '_ + Iterator<Item = Point> {
        self.points.iter().copied()
    }
}
