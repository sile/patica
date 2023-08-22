use crate::command::Command;
use std::collections::VecDeque;

pub trait CommandLog {
    fn command_len(&self) -> usize;
    fn append_command(&mut self, command: Command);
    fn get_redo_command(&self, index: usize) -> Option<&Command>;
    fn get_undo_command(&self, index: usize) -> Option<&Command>;
}

#[derive(Debug, Default, Clone)]
pub struct NullCommandLog {
    len: usize,
}

impl NullCommandLog {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CommandLog for NullCommandLog {
    fn append_command(&mut self, _command: Command) {}

    fn command_len(&self) -> usize {
        self.len
    }

    fn get_redo_command(&self, _index: usize) -> Option<&Command> {
        None
    }

    fn get_undo_command(&self, _index: usize) -> Option<&Command> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct FullCommandLog(LimitedCommandLog);

impl FullCommandLog {
    pub fn new() -> Self {
        Self(LimitedCommandLog::new(usize::MAX))
    }
}

impl Default for FullCommandLog {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandLog for FullCommandLog {
    fn append_command(&mut self, command: Command) {
        self.0.append_command(command);
    }

    fn command_len(&self) -> usize {
        self.0.command_len()
    }

    fn get_redo_command(&self, index: usize) -> Option<&Command> {
        self.0.get_redo_command(index)
    }

    fn get_undo_command(&self, index: usize) -> Option<&Command> {
        self.0.get_undo_command(index)
    }
}

#[derive(Debug, Default, Clone)]
pub struct LimitedCommandLog {
    limit: usize,
    len: usize,
    commands: VecDeque<Command>,
}

impl LimitedCommandLog {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            len: 0,
            commands: VecDeque::new(),
        }
    }
}

impl CommandLog for LimitedCommandLog {
    fn command_len(&self) -> usize {
        self.len
    }

    fn append_command(&mut self, command: Command) {
        self.len += 1;
        self.commands.push_back(command);
        if self.commands.len() > self.limit {
            self.commands.pop_front();
        }
    }

    fn get_redo_command(&self, index: usize) -> Option<&Command> {
        let dropepd = self.len - self.commands.len();
        let i = index.checked_sub(dropepd)?;
        self.commands.get(i)
    }

    fn get_undo_command(&self, index: usize) -> Option<&Command> {
        todo!()
    }
}
