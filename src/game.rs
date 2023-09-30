use crate::{model::Model, screen::Screen, view::View};
use orfail::OrFail;
use pagurus::{
    event::{Event, TimeoutTag},
    image::Canvas,
    video::VideoFrame,
    System,
};
use std::time::Duration;

const TICK_TIMEOUT_TAG: TimeoutTag = TimeoutTag::new(0);

#[derive(Debug)]
pub struct Game {
    model: Model,
    view: View,
    video_frame: VideoFrame,
}

impl Game {
    pub fn new(model: Model) -> Self {
        Self {
            model,
            view: View::default(),
            video_frame: VideoFrame::default(),
        }
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut Model {
        &mut self.model
    }

    fn set_tick_timeout<S: System>(&mut self, system: &mut S, tick_time: Duration) {
        let fps = self.model().fps();
        let elapsed = system.clock_game_time() - tick_time;
        let timeout = Duration::from_secs(1) / u32::from(fps.get());
        let timeout = timeout.saturating_sub(elapsed);
        system.clock_set_timeout(TICK_TIMEOUT_TAG, timeout);
    }

    fn render<S: System>(&mut self, system: &mut S) {
        let size = self.video_frame.spec().resolution;
        let canvas = Canvas::new(&mut self.video_frame);
        let mut screen = Screen::new(canvas, size);
        self.view.render(&self.model, &mut screen);
        system.video_draw(self.video_frame.as_ref());
    }
}

impl<S: System> pagurus::Game<S> for Game {
    fn initialize(&mut self, system: &mut S) -> pagurus::Result<()> {
        self.set_tick_timeout(system, system.clock_game_time());
        Ok(())
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> pagurus::Result<bool> {
        let ticked = match event {
            Event::WindowResized(size) => {
                self.video_frame = VideoFrame::new(system.video_init(size));
                None
            }
            Event::Timeout(TICK_TIMEOUT_TAG) => {
                let now = system.clock_game_time();
                self.model.sync().or_fail()?;
                // TODO: self.model.tick();
                Some(now)
            }
            _ => {
                self.view
                    .handle_event(system, &mut self.model, event)
                    .or_fail()?;
                None
            }
        };
        if let Some(tick_time) = ticked {
            self.render(system);
            self.set_tick_timeout(system, tick_time);
        }
        Ok(!self.model.quit())
    }
}

// #[derive(Debug, Default)]
// pub struct Game {
//     view: View,
//     model: Model,
// }

// impl Game {
//     pub fn set_config(&mut self, config: Config) {
//         self.view.set_key_config(config.key);

//         self.model
//             .apply(&Command::BackgroundColor(config.initial.background_color));
//         for (name, point) in &config.initial.anchors {
//             let command = pati::ImageCommand::anchor(name.clone(), Some(*point));
//             self.model.canvas_mut().apply(&command);
//         }
//     }

//     pub fn model(&self) -> &Model {
//         &self.model
//     }

//     pub fn model_mut(&mut self) -> &mut Model {
//         &mut self.model
//     }

//     }
// }

// impl<S: System> pagurus::Game<S> for Game {
//     fn initialize(&mut self, system: &mut S) -> Result<()> {
//         self.model.initialize().or_fail()?;
//         self.set_tick_timeout(system);
//         Ok(())
//     }

//     fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
//     }
// }
