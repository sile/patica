use crate::{
    color::{Color, Rgba},
    command::{Command, Metadata, MoveDestination, PutCommand, RemoveCommand},
    editor::Editor,
    marker::{MarkKind, Marker},
    spatial::{Point, RectangularArea},
};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone)]
pub struct CanvasStateMachine {
    // TODO: private
    pub cursor: Point,
    pub brush_color: Color,
    pub pixels: Pixels,
    pub metadata: Metadata,
    pub fsm: Fsm,
}

impl CanvasStateMachine {
    pub fn apply(&mut self, command: &Command) -> bool {
        match &command {
            Command::Put(c) => self.handle_put_command(c),
            Command::Remove(c) => self.handle_remove_command(c),
            Command::Mark(c) => self.handle_mark_command(*c),
            Command::Move(c) => self.handle_move_command(c),
            Command::Color => self.handle_color_command(),
        }
    }

    fn handle_color_command(&mut self) -> bool {
        if let Fsm::Marking(m) = &mut self.fsm {
            let color = self.brush_color.to_rgba();
            for point in m.marked_points() {
                self.pixels.0.insert(point, color);
            }
            true
        } else {
            false
        }
    }

    fn handle_move_command(&mut self, dst: &MoveDestination) -> bool {
        match dst {
            MoveDestination::Delta(delta) => {
                let old = self.cursor;
                self.cursor = old + *delta;
                if let Fsm::Marking(m) = &mut self.fsm {
                    m.handle_move(self.cursor);
                }
                self.cursor != old
            }
        }
    }

    fn handle_mark_command(&mut self, kind: MarkKind) -> bool {
        self.fsm = Fsm::new_marking(kind, self);
        true
    }

    fn handle_put_command(&mut self, command: &PutCommand) -> bool {
        match command {
            PutCommand::Metadata(m) => {
                if m.is_empty() {
                    return false;
                }
                for (name, value) in m.iter() {
                    self.metadata.put(name.clone(), value.clone());
                }
            }
        }
        true
    }

    fn handle_remove_command(&mut self, command: &RemoveCommand) -> bool {
        match command {
            RemoveCommand::Metadata(name) => self.metadata.remove(name).is_some(),
        }
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

#[derive(Debug, Default, Clone)]
pub enum Fsm {
    #[default]
    Neutral,
    Marking(Marker),
    Editing(Editor),
}

impl Fsm {
    fn new_marking(kind: MarkKind, machine: &CanvasStateMachine) -> Self {
        Self::Marking(Marker::new(kind, machine))
    }
}
