use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub ticks: u32,
    pub fps: u32,
}

impl Clock {
    pub const DEFAULT_FPS: u32 = 30;

    pub const fn new(fps: u32) -> Self {
        Self { ticks: 0, fps }
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
    }

    pub fn duration(self) -> Duration {
        Duration::from_secs(self.ticks as u64) / self.fps
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new(Self::DEFAULT_FPS)
    }
}
