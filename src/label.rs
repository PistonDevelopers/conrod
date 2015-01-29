use quack::Get;
use graphics;
use color::Color;
use opengl_graphics::Gl;
use internal;
use ui_context::UiContext;
use Position;
use FontSize;
use MaybeColor;
use Text;
use Theme;

/// An enum for passing in label information to widget arguments.
pub enum Labeling<'a> {
    Label(&'a str, internal::FontSize, Color),
    NoLabel,
}

/// Determine the pixel width of the final text bitmap.
#[inline]
pub fn width(uic: &mut UiContext, size: internal::FontSize, text: &str) -> f64 {
    text.chars().fold(0u32, |a, ch| {
        let character = uic.get_character(size, ch);
        a + character.width() as u32
    }) as f64
}

/// Determine a suitable FontSize from a given rectangle height.
#[inline]
pub fn auto_size_from_rect_height(rect_height: f64) -> internal::FontSize {
    let size = rect_height as u32 - 832;
    if size % 2 == 0 { size } else { size - 1u32 }
}

/// A trait used for widget types that take a label.
pub trait Labelable<'a> {
    fn label(self, text: &'a str) -> Self;
    fn label_color(self, color: Color) -> Self;
    fn label_rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self;
    fn label_font_size(self, size: internal::FontSize) -> Self;
    fn small_font(self) -> Self;
    fn medium_font(self) -> Self;
    fn large_font(self) -> Self;
}

////////////////////// NEW DESIGN //////////////////////////////////////////////

/// A label.
#[derive(Copy, Show)]
pub struct Label<'a> {
    text: &'a str,
    pos: internal::Point,
    size: FontSize,
    maybe_color: Option<internal::Color>,
}

impl<'a> Label<'a> {
    /// Creates a new label.
    pub fn new(text: &'a str) -> Label<'a> {
        Label {
            text: text,
            pos: [0.0, 0.0],
            size: FontSize::Medium,
            maybe_color: None,
        }
    }

    pub fn font_size(&self, theme: &Theme) -> internal::FontSize {
        self.size.size(theme)
    }

    pub fn color(&self, theme: &Theme) -> internal::Color {
        self.maybe_color.unwrap_or(theme.label_color.0)
    }

    pub fn draw(&self, uic: &mut UiContext, back_end: &mut Gl) {
        let color = self.maybe_color.unwrap_or(graphics::color::BLACK);
        let size: FontSize = self.get();
        let size = size.size(&uic.theme);
        uic.draw_text(back_end, self.pos, size, Color(color), self.text);
    }
}

quack! {
label: Label['a]
get:
    fn () -> Position { Position(label.pos) }
    fn () -> FontSize { label.size }
    fn () -> MaybeColor { MaybeColor(label.maybe_color) }
    fn () -> Text<'a> { Text(label.text) }
set:
    fn (val: Position) { label.pos = val.0 }
    fn (val: FontSize) { label.size = val }
    fn (val: MaybeColor) { label.maybe_color = val.0 }
    fn (val: Color) { label.maybe_color = Some(val.0) }
    fn (val: Text<'a>) { label.text = val.0 }
action:
}
