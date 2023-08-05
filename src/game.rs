use pagurus::{
    event::Event,
    image::{Canvas, Color},
    video::VideoFrame,
    Result, System,
};

#[derive(Debug, Default)]
pub struct Game {
    video_frame: VideoFrame,
}

impl<S: System> pagurus::Game<S> for Game {
    fn initialize(&mut self, _system: &mut S) -> Result<()> {
        Ok(())
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
        match event {
            Event::WindowResized(size) => {
                self.video_frame = VideoFrame::new(system.video_init(size));
                let mut canvas = Canvas::new(&mut self.video_frame);
                canvas.fill_color(Color::RED);
                system.video_draw(self.video_frame.as_ref());
            }
            _ => {}
        }
        Ok(true)
    }
}
