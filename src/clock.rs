use serde::{Deserialize, Serialize};
use std::{num::NonZeroU8, time::Duration};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Ticks(u32);

impl Ticks {
    pub const fn new(n: u32) -> Self {
        Self(n)
    }

    pub const fn get(self) -> u32 {
        self.0
    }

    pub fn tick_delta(&mut self, delta: i32) {
        if delta < 0 {
            self.0 = self.0.saturating_sub(delta.unsigned_abs());
        } else {
            self.0 = self.0.saturating_add(delta as u32);
        }
    }

    pub fn tick(&mut self) {
        self.tick_delta(1);
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Time {
    pub ticks: Ticks,
    pub fps: NonZeroU8, // TODO: Fps
}

impl Time {
    pub const DEFAULT_FPS: u8 = 30;

    pub const fn new(ticks: Ticks, fps: NonZeroU8) -> Self {
        Self { ticks, fps }
    }

    pub fn duration(self) -> Duration {
        Duration::from_secs(self.ticks.0 as u64) / self.fps.get() as u32
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new(
            Ticks::new(0),
            NonZeroU8::new(Self::DEFAULT_FPS).expect("unreachable"),
        )
    }
}
