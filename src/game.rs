use crate::constants::{COLOR_BG0, COLOR_BG1};
use pagurus::{event::Event, image::Canvas, video::VideoFrame, Result, System};

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
                Self::render_background(&mut canvas);
                system.video_draw(self.video_frame.as_ref());
            }
            _ => {}
        }
        Ok(true)
    }
}

impl Game {
    fn render_background(canvas: &mut Canvas) {
        for position in canvas.drawing_region().iter() {
            let color = if (position.x + position.y) % 2 == 0 {
                COLOR_BG0
            } else {
                COLOR_BG1
            };
            canvas.draw_pixel(position, color);
        }
    }
}
