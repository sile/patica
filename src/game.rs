use crate::{
    config::Config,
    model::Model,
    view::{View, ViewContext},
};
use pagurus::{
    event::{Event, TimeoutTag},
    failure::OrFail,
    image::Canvas,
    video::VideoFrame,
    Result, System,
};
use std::num::NonZeroU64;
use std::time::Duration;

const FPS: u32 = 30;
const RENDER_TIMEOUT_TAG: TimeoutTag = TimeoutTag::new(0);

// TODO: delete
#[derive(Debug, Clone, Copy)]
pub struct GameClock {
    pub ticks: u64,
    pub fps: NonZeroU64,
}

impl GameClock {
    pub const fn new(fps: NonZeroU64) -> Self {
        Self { ticks: 0, fps }
    }

    pub fn tick(&mut self) {
        self.ticks += 1;
    }
}

impl Default for GameClock {
    fn default() -> Self {
        Self::new(NonZeroU64::new(30).expect("unreachable"))
    }
}

#[derive(Debug, Default)]
pub struct Game {
    video_frame: VideoFrame,
    view: View,
    model: Model,
    clock: GameClock,
}

impl Game {
    pub fn set_config(&mut self, config: Config) {
        self.view.set_key_config(config.key);
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut Model {
        &mut self.model
    }

    fn render<S: System>(&mut self, system: &mut S) {
        let ctx = self.view_context(system);
        let mut canvas = Canvas::new(&mut self.video_frame);
        self.view.render(&ctx, &self.model, &mut canvas);
        system.video_draw(self.video_frame.as_ref());
    }

    fn view_context<S: System>(&self, _system: &S) -> ViewContext {
        ViewContext {
            window_size: self.video_frame.spec().resolution,
            quit: false,
        }
    }
}

impl<S: System> pagurus::Game<S> for Game {
    fn initialize(&mut self, system: &mut S) -> Result<()> {
        system.clock_set_timeout(RENDER_TIMEOUT_TAG, Duration::from_secs(1) / FPS);
        Ok(())
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
        match event {
            Event::WindowResized(size) => {
                self.video_frame = VideoFrame::new(system.video_init(size));
                self.render(system);
            }
            Event::Timeout(RENDER_TIMEOUT_TAG) => {
                self.clock.tick();
                self.render(system);
                system.clock_set_timeout(RENDER_TIMEOUT_TAG, Duration::from_secs(1) / FPS);
                return Ok(true);
            }
            _ => {}
        }

        let mut ctx = self.view_context(system);
        self.view
            .handle_event(&mut ctx, &mut self.model, event)
            .or_fail()?;
        Ok(!ctx.quit)
    }
}
