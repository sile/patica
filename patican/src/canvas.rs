use crate::{
    canvas_state_machine::{CanvasStateMachine, Pixels},
    color::Color,
    command::{Command, Metadata},
    log::CommandLog,
    spatial::Point,
};

#[derive(Debug, Clone)]
pub struct Canvas<L> {
    machine: CanvasStateMachine,
    log: L,
}

impl<L: Default> Canvas<L> {
    pub fn new() -> Self {
        Self {
            machine: CanvasStateMachine::default(),
            log: L::default(),
        }
    }
}

impl<L> Canvas<L> {
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

    pub fn history(&self) -> &L {
        &self.log
    }

    pub fn history_mut(&mut self) -> &mut L {
        &mut self.log
    }
}

impl<L: CommandLog> Canvas<L> {
    pub fn apply(&mut self, command: Command) -> bool {
        let applied = self.machine.apply(&command);
        if applied {
            self.log.append_command(command, &self.machine);
        }
        applied
    }
}

impl<H: Default> Default for Canvas<H> {
    fn default() -> Self {
        Self::new()
    }
}
