use crate::records::Record;
use pagurus::spatial::{Position, Size};

#[derive(Debug, Default)]
pub struct Model {
    cursor: Position,
    // palette
    // pixels
}

impl Model {
    pub fn cursor(&self) -> Position {
        self.cursor
    }

    pub fn on_canvas_resize(&mut self, _size: Size) {
        todo!()
    }

    pub fn handle_record(&mut self, _record: &Record) -> pagurus::Result<()> {
        todo!()
    }
}
