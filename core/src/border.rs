
use color::{Color, hsl, hsla, rgb, rgba};

/// To be used as a parameter for defining the aesthetic
/// of the widget border.
#[derive(Copy, Clone)]
pub enum Bordering {
    /// Border width and color.
    Border(f64, Color),
    /// No border.
    NoBorder,
}

/// Widgets that may display a border.
pub trait Borderable: Sized {

    /// Set the width of the widget's border.
    fn border(self, width: f64) -> Self;

    /// Set the color of the widget's border.
    fn border_color(self, color: Color) -> Self;

    /// Set the color of the widget's border with rgba values.
    fn border_rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.border_color(rgba(r, g, b, a))
    }

    /// Set the color of the widget's border with rgb values.
    fn border_rgb(self, r: f32, g: f32, b: f32) -> Self {
        self.border_color(rgb(r, g, b))
    }

    /// Set the color of the widget's border with hsla values.
    fn border_hsla(self, h: f32, s: f32, l: f32, a: f32) -> Self {
        self.border_color(hsla(h, s, l, a))
    }

    /// Set the color of the widget's border with hsl values.
    fn border_hsl(self, h: f32, s: f32, l: f32) -> Self {
        self.border_color(hsl(h, s, l))
    }

}

