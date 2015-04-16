
use color::{Color, hsl, hsla, rgb, rgba};
use graphics::character::CharacterCache;
use ui::Ui;

pub type FontSize = u32;

/// Determine the pixel width of the final text bitmap.
#[inline]
pub fn width<C: CharacterCache>(ui: &mut Ui<C>, size: FontSize, text: &str) -> f64 {
    text.chars().fold(0u32, |a, ch| {
        let character = ui.get_character(size, ch);
        a + character.width() as u32
    }) as f64
}

/// Widgets that may display some label.
pub trait Labelable<'a>: Sized {

    /// Set the label for the widget.
    fn label(self, text: &'a str) -> Self;

    /// Set the color of the widget's label.
    fn label_color(self, color: Color) -> Self;

    /// Set the color of the widget's label from rgba values.
    fn label_rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.label_color(rgba(r, g, b, a))
    }

    /// Set the color of the widget's label from rgb values.
    fn label_rgb(self, r: f32, g: f32, b: f32) -> Self {
        self.label_color(rgb(r, g, b))
    }

    /// Set the color of the widget's label from hsla values.
    fn label_hsla(self, h: f32, s: f32, l: f32, a: f32) -> Self {
        self.label_color(hsla(h, s, l, a))
    }

    /// Set the color of the widget's label from hsl values.
    fn label_hsl(self, h: f32, s: f32, l: f32) -> Self {
        self.label_color(hsl(h, s, l))
    }

    /// Set the font size for the widget's label.
    fn label_font_size(self, size: FontSize) -> Self;

    /// Set a "small" font size for the widget's label.
    fn small_font<C>(self, ui: &Ui<C>) -> Self {
        self.label_font_size(ui.theme.font_size_small)
    }

    /// Set a "medium" font size for the widget's label.
    fn medium_font<C>(self, ui: &Ui<C>) -> Self {
        self.label_font_size(ui.theme.font_size_medium)
    }

    /// Set a "large" font size for the widget's label.
    fn large_font<C>(self, ui: &Ui<C>) -> Self {
        self.label_font_size(ui.theme.font_size_large)
    }

}

