use crate::{
    config::Config,
    model::{GameClock, Model},
    view::{View, ViewContext},
};
use pagurus::{
    event::{Event, TimeoutTag},
    failure::OrFail,
    image::Canvas,
    video::VideoFrame,
    Result, System,
};
use std::sync::Arc;
use std::time::Duration;

const FPS: u32 = 30;
const RENDER_TIMEOUT_TAG: TimeoutTag = TimeoutTag::new(0);

#[derive(Debug, Default)]
pub struct Game {
    video_frame: VideoFrame,
    view: View,
    model: Option<Model>,
    config: Arc<Config>,
    clock: GameClock,
}

impl Game {
    pub fn set_model(&mut self, model: Model) {
        self.model = Some(model);
    }

    pub fn take_model(&mut self) -> Option<Model> {
        self.model.take()
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = Arc::new(config);
    }

    fn render<S: System>(&mut self, system: &mut S) -> pagurus::Result<()> {
        let ctx = self.make_view_context(system).or_fail()?;
        let mut canvas = Canvas::new(&mut self.video_frame);
        self.view.render(&ctx, &mut canvas);
        system.video_draw(self.video_frame.as_ref());
        self.model = Some(ctx.model);
        Ok(())
    }

    fn make_view_context<S: System>(&mut self, system: &S) -> pagurus::Result<ViewContext> {
        Ok(ViewContext {
            window_size: self.video_frame.spec().resolution,
            now: system.clock_game_time(),
            model: self.model.take().or_fail()?,
            config: Arc::clone(&self.config),
            quit: false,
            clock: self.clock,
        })
    }
}

impl<S: System> pagurus::Game<S> for Game {
    fn initialize(&mut self, system: &mut S) -> Result<()> {
        self.model = Some(Model::default());
        system.clock_set_timeout(RENDER_TIMEOUT_TAG, Duration::from_secs(1) / FPS);
        Ok(())
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
        match event {
            Event::WindowResized(size) => {
                self.video_frame = VideoFrame::new(system.video_init(size));
                self.render(system).or_fail()?;
            }
            Event::Timeout(RENDER_TIMEOUT_TAG) => {
                self.clock.tick();
                self.render(system).or_fail()?;
                system.clock_set_timeout(RENDER_TIMEOUT_TAG, Duration::from_secs(1) / FPS);
                return Ok(true);
            }
            _ => {}
        }

        let mut ctx = self.make_view_context(system).or_fail()?;
        self.view.handle_event(&mut ctx, event).or_fail()?;
        self.model = Some(ctx.model);

        Ok(!ctx.quit)
    }
}
