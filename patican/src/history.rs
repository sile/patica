use std::collections::VecDeque;

use crate::command::Command;

pub trait History {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn append_command(&mut self, command: Command);
    fn get_redo_command(&self, index: usize) -> Option<&Command>;
    fn get_undo_command(&self, index: usize) -> Option<&Command>;
}

#[derive(Debug, Default, Clone)]
pub struct NullHistory {
    len: usize,
}

impl NullHistory {
    pub fn new() -> Self {
        Self::default()
    }
}

impl History for NullHistory {
    fn append_command(&mut self, _command: Command) {}

    fn len(&self) -> usize {
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
pub struct FullHistory(LimitedHistory);

impl FullHistory {
    pub fn new() -> Self {
        Self(LimitedHistory::new(usize::MAX))
    }
}

impl Default for FullHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl History for FullHistory {
    fn append_command(&mut self, command: Command) {
        self.0.append_command(command);
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn get_redo_command(&self, index: usize) -> Option<&Command> {
        self.0.get_redo_command(index)
    }

    fn get_undo_command(&self, index: usize) -> Option<&Command> {
        self.0.get_undo_command(index)
    }
}

#[derive(Debug, Default, Clone)]
pub struct LimitedHistory {
    limit: usize,
    len: usize,
    commands: VecDeque<Command>,
}

impl LimitedHistory {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            len: 0,
            commands: VecDeque::new(),
        }
    }
}

impl History for LimitedHistory {
    fn len(&self) -> usize {
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
