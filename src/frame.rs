
use color::{Color, hsl, hsla, rgb, rgba};

/// To be used as a parameter for defining the aesthetic
/// of the widget frame.
#[derive(Copy, Clone)]
pub enum Framing {
    /// Frame width and color.
    Frame(f64, Color),
    /// No frame.
    NoFrame,
}

/// Widgets that may display a frame.
pub trait Frameable: Sized {

    /// Set the width of the widget's frame.
    fn frame(self, width: f64) -> Self;

    /// Set the color of the widget's frame.
    fn frame_color(self, color: Color) -> Self;

    /// Set the color of the widget's frame with rgba values.
    fn frame_rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.frame_color(rgba(r, g, b, a))
    }

    /// Set the color of the widget's frame with rgb values.
    fn frame_rgb(self, r: f32, g: f32, b: f32) -> Self {
        self.frame_color(rgb(r, g, b))
    }

    /// Set the color of the widget's frame with hsla values.
    fn frame_hsla(self, h: f32, s: f32, l: f32, a: f32) -> Self {
        self.frame_color(hsla(h, s, l, a))
    }

    /// Set the color of the widget's frame with hsl values.
    fn frame_hsl(self, h: f32, s: f32, l: f32) -> Self {
        self.frame_color(hsl(h, s, l))
    }

}

