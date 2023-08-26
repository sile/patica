use crate::{
    command::Command,
    config::Config,
    model::Model,
    view::{View, WindowCanvas},
};
use pagurus::{
    event::{Event, TimeoutTag},
    failure::OrFail,
    image::Canvas,
    video::VideoFrame,
    Result, System,
};
use std::time::Duration;

const TICK_TIMEOUT_TAG: TimeoutTag = TimeoutTag::new(0);

#[derive(Debug, Default)]
pub struct Game {
    video_frame: VideoFrame,
    view: View,
    model: Model,
}

impl Game {
    pub fn set_config(&mut self, config: Config) {
        self.view.set_key_config(config.key);

        self.model
            .apply(&Command::BackgroundColor(config.initial.background_color));
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut Model {
        &mut self.model
    }

    fn render<S: System>(&mut self, system: &mut S) {
        let size = self.video_frame.spec().resolution;
        let mut canvas = WindowCanvas::new(Canvas::new(&mut self.video_frame), size);
        self.view.render(&self.model, &mut canvas);
        system.video_draw(self.video_frame.as_ref());
    }

    fn set_tick_timeout<S: System>(&mut self, system: &mut S) {
        system.clock_set_timeout(TICK_TIMEOUT_TAG, Duration::from_secs(1) / self.model.fps());
    }
}

impl<S: System> pagurus::Game<S> for Game {
    fn initialize(&mut self, system: &mut S) -> Result<()> {
        self.model.initialize().or_fail()?;
        self.set_tick_timeout(system);
        Ok(())
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
        let mut set_timeout = false;
        match event {
            Event::WindowResized(size) => {
                self.video_frame = VideoFrame::new(system.video_init(size));
            }
            Event::Timeout(TICK_TIMEOUT_TAG) => {
                self.model.tick();
                set_timeout = true;
            }
            _ => {}
        }
        self.view
            .handle_event(system, &mut self.model, event)
            .or_fail()?;
        self.render(system);
        if set_timeout {
            self.set_tick_timeout(system);
        }
        Ok(!self.model.quit())
    }
}
