use pagurus::{image::Canvas, spatial::Size};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Screen<'a> {
    canvas: Canvas<'a>,
    screen_size: Size,
}

impl<'a> Screen<'a> {
    pub fn new(canvas: Canvas<'a>, screen_size: Size) -> Self {
        Self {
            canvas,
            screen_size,
        }
    }
}
