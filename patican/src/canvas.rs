use crate::{
    canvas_state_machine::{CanvasStateMachine, Pixels},
    color::Color,
    command::{Command, Metadata},
    log::CommandLog,
    marker::Marker,
    spatial::Point,
};
use std::io::Read;

#[derive(Debug, Clone)]
pub struct Canvas<L> {
    machine: CanvasStateMachine,
    log: L,
}

impl<L: CommandLog + Default> Canvas<L> {
    pub fn new() -> Self {
        Self {
            machine: CanvasStateMachine::default(),
            log: L::default(),
        }
    }

    pub fn load<R: Read>(log_reader: R) -> serde_json::Result<Self> {
        let mut canvas = Self::new();
        for command in serde_json::Deserializer::from_reader(log_reader).into_iter::<Command>() {
            let command = command?;
            canvas.apply(command);
        }
        Ok(canvas)
    }

    pub fn cursor(&self) -> Point {
        self.machine.cursor
    }

    pub fn brush_color(&self) -> Color {
        self.machine.brush_color
    }

    pub fn metadata(&self) -> &Metadata {
        &self.machine.metadata
    }

    pub fn pixels(&self) -> &Pixels {
        &self.machine.pixels
    }

    pub fn log(&self) -> &L {
        &self.log
    }

    pub fn log_mut(&mut self) -> &mut L {
        &mut self.log
    }

    pub fn marker(&self) -> Option<&Marker> {
        self.machine.fsm.as_marker()
    }

    pub fn apply(&mut self, command: Command) -> bool {
        let applied = self.machine.apply(&command);
        if applied {
            self.log.append_command(command, &self.machine);
        }
        applied
    }
}

impl<H: CommandLog + Default> Default for Canvas<H> {
    fn default() -> Self {
        Self::new()
    }
}
