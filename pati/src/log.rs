use crate::Canvas;
use crate::Command;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Version(pub(crate) u32);

impl std::ops::Add<u32> for Version {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0.saturating_add(rhs))
    }
}

impl std::ops::Sub<u32> for Version {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self::Output {
        Self(self.0.saturating_sub(rhs))
    }
}

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

    pub fn commands(&self) -> &[Command] {
        &self.commands
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Canvas, Color, PatchCommand, PatchEntry, Point};

    #[test]
    fn restore_canvas_works() {
        let mut canvas = Canvas::new();
        let mut log = Log::default();
        assert_eq!(log.latest_canvas_version(), Version(0));

        let color = Color::rgb(100, 0, 0);
        let entry = PatchEntry {
            color: Some(color),
            points: vec![Point::new(1, 3)],
        };
        let command = Command::Patch(PatchCommand::new(vec![entry]));
        assert!(canvas.apply(&command));
        log.append_applied_command(command, &canvas);
        assert_eq!(log.latest_canvas_version(), Version(1));

        let old_canvas = log.restore_canvas(Version(0)).unwrap();
        assert_ne!(old_canvas.pixels().len(), canvas.pixels().len());
    }
}
