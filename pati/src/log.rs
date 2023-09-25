use crate::Image;
use crate::ImageCommand;
use serde::{Deserialize, Serialize};

/// Number of applied commands.
// TODO: s/Version/ImageVersion/
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
    commands: Vec<ImageCommand>,
    snapshots: Vec<Snapshot>,
}

impl Log {
    pub fn latest_image_version(&self) -> Version {
        Version(self.commands.len() as u32)
    }

    pub fn append_applied_command(&mut self, command: ImageCommand, image: &Image) {
        self.commands.push(command);
        if self.commands.len() % 1000 == 0 {
            self.snapshots.push(Snapshot {
                version: Version(self.commands.len() as u32),
                image: image.clone(),
            });
        }
    }

    pub fn commands(&self) -> &[ImageCommand] {
        &self.commands
    }

    pub fn restore_image(&self, version: Version) -> Option<Image> {
        if self.latest_image_version() < version {
            return None;
        }

        match self.snapshots.binary_search_by_key(&version, |s| s.version) {
            Ok(i) => Some(self.snapshots[i].image.clone()),
            Err(i) => {
                let mut snapshot = self.snapshots[i - 1].clone();
                for i in snapshot.version.0..version.0 {
                    snapshot.image.apply(&self.commands[i as usize]);
                }
                Some(snapshot.image)
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
    image: Image,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Image, PatchEntry, PatchImageCommand, Point};

    #[test]
    fn restore_image_works() {
        let mut image = Image::new();
        let mut log = Log::default();
        assert_eq!(log.latest_image_version(), Version(0));

        let color = Color::rgb(100, 0, 0);
        let entry = PatchEntry {
            color: Some(color),
            points: vec![Point::new(1, 3)],
        };
        let command = ImageCommand::Patch(PatchImageCommand::new(vec![entry]));
        assert!(image.apply(&command));
        log.append_applied_command(command, &image);
        assert_eq!(log.latest_image_version(), Version(1));

        let old_image = log.restore_image(Version(0)).unwrap();
        assert_ne!(old_image.pixels().len(), image.pixels().len());
    }
}
