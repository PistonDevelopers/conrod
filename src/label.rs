
use color::{rgba, Color};
use graphics::character::CharacterCache;
use ui::Ui;

pub type FontSize = u32;

/// An enum for passing in label information to widget arguments.
pub enum Labeling<'a> {
    Label(&'a str, FontSize, Color),
    NoLabel,
}

/// Determine the pixel width of the final text bitmap.
#[inline]
pub fn width<C: CharacterCache>(ui: &mut Ui<C>, size: FontSize, text: &str) -> f64 {
    text.chars().fold(0u32, |a, ch| {
        let character = ui.get_character(size, ch);
        a + character.width() as u32
    }) as f64
}

/// Determine a suitable FontSize from a given rectangle height.
#[inline]
pub fn auto_size_from_rect_height(rect_height: f64) -> FontSize {
    let size = rect_height as u32 - 832;
    if size % 2 == 0 { size } else { size - 1u32 }
}

/// A trait used for widget types that take a label.
pub trait Labelable<'a>: Sized {
    fn label(self, text: &'a str) -> Self;
    fn label_color(self, color: Color) -> Self;
    fn label_rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.label_color(rgba(r, g, b, a))
    }
    fn label_font_size(self, size: FontSize) -> Self;
    fn small_font<C>(self, ui: &Ui<C>) -> Self {
        self.label_font_size(ui.theme.font_size_small)
    }
    fn medium_font<C>(self, ui: &Ui<C>) -> Self {
        self.label_font_size(ui.theme.font_size_medium)
    }
    fn large_font<C>(self, ui: &Ui<C>) -> Self {
        self.label_font_size(ui.theme.font_size_large)
    }
}

