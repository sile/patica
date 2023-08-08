use crate::{
    model_actor::ModelActorHandle,
    view::{View, ViewContext},
};
use pagurus::{
    event::{Event, TimeoutTag},
    failure::OrFail,
    image::Canvas,
    video::VideoFrame,
    Result, System,
};
use std::time::Duration;

const FPS: u32 = 30;
const RENDER_TIMEOUT_TAG: TimeoutTag = TimeoutTag::new(0);

#[derive(Debug, Default)]
pub struct Game {
    video_frame: VideoFrame,
    model: ModelActorHandle,
    view: View,
}

impl Game {
    fn render<S: System>(&mut self, system: &mut S) {
        let size = self.video_frame.spec().resolution;
        let mut canvas = Canvas::new(&mut self.video_frame);
        let ctx = ViewContext::new(size, system.clock_game_time(), self.model.clone());
        self.view.render(&ctx, &mut canvas);
        system.video_draw(self.video_frame.as_ref());
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
                self.render(system);
                system.clock_set_timeout(RENDER_TIMEOUT_TAG, Duration::from_secs(1) / FPS);
                return Ok(true);
            }
            _ => {}
        }

        let ctx = ViewContext::new(
            self.video_frame.spec().resolution,
            system.clock_game_time(),
            self.model.clone(),
        );
        self.view.handle_event(&ctx, event).or_fail()?;

        Ok(true)
    }
}
