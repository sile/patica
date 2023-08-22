use crate::canvas_state_machine::CanvasStateMachine;
use crate::command::Command;
use std::collections::{BTreeMap, VecDeque};

pub trait CommandLog {
    fn command_len(&self) -> usize;
    fn append_command(&mut self, command: Command, machine: &CanvasStateMachine);
    fn get_command(&self, index: usize) -> Option<&Command>;
    fn restore_machine(&self, index: usize) -> Option<CanvasStateMachine>;
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
    fn append_command(&mut self, _command: Command, _machine: &CanvasStateMachine) {}

    fn command_len(&self) -> usize {
        self.len
    }

    fn get_command(&self, _index: usize) -> Option<&Command> {
        None
    }

    fn restore_machine(&self, _index: usize) -> Option<CanvasStateMachine> {
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
    fn append_command(&mut self, command: Command, machine: &CanvasStateMachine) {
        self.0.append_command(command, machine);
    }

    fn command_len(&self) -> usize {
        self.0.command_len()
    }

    fn get_command(&self, index: usize) -> Option<&Command> {
        self.0.get_command(index)
    }

    fn restore_machine(&self, index: usize) -> Option<CanvasStateMachine> {
        self.0.restore_machine(index)
    }
}

#[derive(Debug, Default, Clone)]
pub struct LimitedCommandLog {
    limit: usize,
    snapshot_interval: usize,
    len: usize,
    commands: VecDeque<Command>,
    snapshots: BTreeMap<usize, CanvasStateMachine>,
}

impl LimitedCommandLog {
    fn first_snapshot_index(&self) -> usize {
        self.snapshots
            .first_key_value()
            .map(|(k, _)| *k)
            .unwrap_or(0)
    }

    fn second_snapshot_index(&self) -> usize {
        self.snapshots.keys().take(2).last().copied().unwrap_or(0)
    }

    fn last_snapshot_index(&self) -> usize {
        self.snapshots
            .last_key_value()
            .map(|(k, _)| *k)
            .unwrap_or(0)
    }

    fn first_command_index(&self) -> usize {
        self.len - self.commands.len()
    }
}

impl LimitedCommandLog {
    pub fn new(limit: usize) -> Self {
        let limit = limit.max(100);
        Self {
            limit,
            snapshot_interval: (limit / 2).min(1000), // TODO
            len: 0,
            commands: VecDeque::new(),
            snapshots: [(0, CanvasStateMachine::default())].into_iter().collect(),
        }
    }
}

impl CommandLog for LimitedCommandLog {
    fn command_len(&self) -> usize {
        self.len
    }

    fn append_command(&mut self, command: Command, machine: &CanvasStateMachine) {
        self.len += 1;
        self.commands.push_back(command);

        if self.last_snapshot_index() + self.snapshot_interval < self.len {
            self.snapshots.insert(self.len, machine.clone());
        }

        while self.second_snapshot_index() < self.len.saturating_sub(self.limit) {
            self.snapshots.pop_first();
        }

        while self.first_command_index() < self.first_snapshot_index() {
            self.commands.pop_front();
        }
    }

    fn get_command(&self, index: usize) -> Option<&Command> {
        let i = index.checked_sub(self.first_command_index())?;
        self.commands.get(i)
    }

    fn restore_machine(&self, index: usize) -> Option<CanvasStateMachine> {
        let (snapshot_index, mut machine) = self
            .snapshots
            .range(..=index)
            .last()
            .map(|(i, m)| (*i, m.clone()))?;
        for i in snapshot_index..index {
            let command = self.get_command(i)?;
            machine.apply(command);
        }
        Some(machine)
    }
}
