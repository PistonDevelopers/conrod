use piston::quack::{ Pair, Set, SetAt };

use color::Color;

/// To be used as a parameter for defining the aesthetic
/// of the widget frame.
#[derive(Copy)]
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

/// Frame width property.
#[derive(Copy)]
pub struct FrameWidth(pub f64);

/// Frame color property.
#[derive(Copy)]
pub struct FrameColor(pub Color);

impl<T> Frameable for T
    where
        (FrameWidth, T): Pair<Data = FrameWidth, Object = T> + SetAt,
        (FrameColor, T): Pair<Data = FrameColor, Object = T> + SetAt
{
    fn frame(self, width: f64) -> Self {
        self.set(FrameWidth(width))
    }

    fn frame_color(self, color: Color) -> Self {
        self.set(FrameColor(color))
    }

    fn frame_rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.set(FrameColor(Color([r, g, b, a])))
    }
}
