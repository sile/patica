use crate::canvas::CanvasState;
use crate::Command;
use serde::{Deserialize, Serialize};

pub trait Log: Default {
    fn latest_state_version(&self) -> Version;
    fn append_applied_command(&mut self, command: Command, state: &CanvasState);
    fn restore_state(&self, version: Version) -> Option<CanvasState>;
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Version(u32);

#[derive(Debug, Default, Clone)]
pub struct NullLog {
    version: Version,
}

impl Log for NullLog {
    fn latest_state_version(&self) -> Version {
        self.version
    }

    fn append_applied_command(&mut self, _command: Command, _state: &CanvasState) {
        self.version.0 += 1;
    }

    fn restore_state(&self, _version: Version) -> Option<CanvasState> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct FullLog {
    commands: Vec<Command>,
    snapshots: Vec<Snapshot>,
}

impl Log for FullLog {
    fn latest_state_version(&self) -> Version {
        Version(self.commands.len() as u32)
    }

    fn append_applied_command(&mut self, command: Command, state: &CanvasState) {
        self.commands.push(command);
        if self.commands.len() % 1000 == 0 {
            self.snapshots.push(Snapshot {
                version: Version(self.commands.len() as u32),
                state: state.clone(),
            });
        }
    }

    fn restore_state(&self, version: Version) -> Option<CanvasState> {
        if self.latest_state_version() < version {
            return None;
        }

        match self.snapshots.binary_search_by_key(&version, |s| s.version) {
            Ok(i) => Some(self.snapshots[i].state.clone()),
            Err(i) => {
                let mut snapshot = self.snapshots[i - 1].clone();
                for i in snapshot.version.0..version.0 {
                    snapshot.state.apply(&self.commands[i as usize]);
                }
                Some(snapshot.state)
            }
        }
    }
}

impl Default for FullLog {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            snapshots: vec![Snapshot::default()],
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Snapshot {
    version: Version,
    state: CanvasState,
}
