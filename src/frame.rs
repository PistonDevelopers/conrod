
use color::Color;

/// To be used as a parameter for defining the aesthetic
/// of the widget frame.
#[deriving(Copy)]
pub enum Framing {
    /// Frame width and color.
    Frame(f64, Color),
    NoFrame,
}

/// A trait used for "colorable" widget context types.
pub trait Frameable {
    fn frame(self, width: f64) -> Self;
    fn frame_color(self, color: Color) -> Self;
    fn frame_rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self;
}

