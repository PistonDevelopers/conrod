
use color::{Color, rgba};

/// To be used as a parameter for defining the aesthetic
/// of the widget frame.
#[derive(Copy, Clone)]
pub enum Framing {
    /// Frame width and color.
    Frame(f64, Color),
    NoFrame,
}

/// A trait used for "colorable" widget context types.
pub trait Frameable: Sized {
    fn frame(self, width: f64) -> Self;
    fn frame_color(self, color: Color) -> Self;
    fn frame_rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.frame_color(rgba(r, g, b, a))
    }
}
