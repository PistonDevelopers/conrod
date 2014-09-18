
use color::Color;

/// To be used as a parameter for defining the aesthetic
/// of the widget frame.
pub enum Framing {
    /// Frame width and color.
    Frame(f64, Color),
    NoFrame,
}

/// A trait used for "colorable" widget context types.
pub trait Frameable {
    fn frame(self, width: f64, color: Color) -> Self;
}

