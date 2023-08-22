use serde::{Deserialize, Serialize};

use crate::{canvas_state_machine::CanvasStateMachine, spatial::Point};

#[derive(Debug, Clone)]
pub enum Marker {
    Line(LineMarker),
}

impl Marker {
    pub fn new(kind: MarkKind, machine: &CanvasStateMachine) -> Self {
        match kind {
            MarkKind::Line => Self::Line(LineMarker::new(machine)),
        }
    }

    pub fn handle_move(&mut self, cursor: Point) {
        match self {
            Self::Line(marker) => marker.handle_move(cursor),
        }
    }

    pub fn marked_points(&self) -> Box<dyn '_ + Iterator<Item = Point>> {
        match self {
            Self::Line(marker) => Box::new(marker.marked_points()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkKind {
    Line,
}

#[derive(Debug, Clone)]
pub struct LineMarker {
    start: Point,
    end: Point,
}

impl LineMarker {
    fn new(machine: &CanvasStateMachine) -> Self {
        Self {
            start: machine.cursor,
            end: machine.cursor,
        }
    }

    fn handle_move(&mut self, cursor: Point) {
        self.end = cursor;
    }

    fn marked_points(&self) -> impl Iterator<Item = Point> {
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

fn xy(x: i16, y: i16) -> Point {
    Point::new(x, y)
}

fn yx(y: i16, x: i16) -> Point {
    Point::new(x, y)
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
