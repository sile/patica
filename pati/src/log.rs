use crate::Canvas;
use crate::Command;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Version(u32);

#[derive(Debug, Clone)]
pub struct Log {
    commands: Vec<Command>,
    snapshots: Vec<Snapshot>,
}

impl Log {
    pub fn latest_canvas_version(&self) -> Version {
        Version(self.commands.len() as u32)
    }

    pub fn append_applied_command(&mut self, command: Command, canvas: &Canvas) {
        self.commands.push(command);
        if self.commands.len() % 1000 == 0 {
            self.snapshots.push(Snapshot {
                version: Version(self.commands.len() as u32),
                canvas: canvas.clone(),
            });
        }
    }

    pub fn restore_canvas(&self, version: Version) -> Option<Canvas> {
        if self.latest_canvas_version() < version {
            return None;
        }

        match self.snapshots.binary_search_by_key(&version, |s| s.version) {
            Ok(i) => Some(self.snapshots[i].canvas.clone()),
            Err(i) => {
                let mut snapshot = self.snapshots[i - 1].clone();
                for i in snapshot.version.0..version.0 {
                    snapshot.canvas.apply(&self.commands[i as usize]);
                }
                Some(snapshot.canvas)
            }
        }
    }
}

impl Default for Log {
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
    canvas: Canvas,
}
