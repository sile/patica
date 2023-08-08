use crate::{
    model::Model,
    view::{View, ViewContext},
};
use pagurus::{
    event::{Event, TimeoutTag},
    failure::{Failure, OrFail},
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
    view: View,
    model: Option<Model>,
}

impl Game {
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
                self.render(system).or_fail()?;
                system.clock_set_timeout(RENDER_TIMEOUT_TAG, Duration::from_secs(1) / FPS);
                return Ok(true);
            }
            _ => {}
        }

        let mut ctx = self.make_view_context(system).or_fail()?;
        self.view.handle_event(&mut ctx, event).or_fail()?;
        self.model = Some(ctx.model);

        Ok(true)
    }

    fn query(&mut self, _system: &mut S, name: &str) -> Result<Vec<u8>> {
        match name {
            "model.take_applied_commands" => {
                let commands = self.model.as_mut().or_fail()?.take_applied_commands();
                let data = serde_json::to_vec(&commands).or_fail()?;
                Ok(data)
            }
            _ => Err(Failure::new().message(format!("unknown query: {name:?}"))),
        }
    }

    fn command(&mut self, _system: &mut S, name: &str, data: &[u8]) -> Result<()> {
        match name {
            "model.apply_command" => {
                let command = serde_json::from_slice(data).or_fail()?;
                self.model.as_mut().or_fail()?.apply(command).or_fail()?;
                Ok(())
            }
            _ => Err(Failure::new().message(format!("unknown command: {name:?}"))),
        }
    }
}
