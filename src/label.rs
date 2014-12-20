
use color::Color;
use opengl_graphics::Gl;
use point::Point;
use ui_context::UiContext;

pub type FontSize = u32;

/// An enum for passing in label information to widget arguments.
pub enum Labeling<'a> {
    Label(&'a str, FontSize, Color),
    NoLabel,
}

/// Determine the pixel width of the final text bitmap.
#[inline]
pub fn width(uic: &mut UiContext, size: FontSize, text: &str) -> f64 {
    text.chars().fold(0u32, |a, ch| {
        let character = uic.get_character(size, ch);
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
pub trait Labelable<'a> {
    fn label(self, text: &'a str) -> Self;
    fn label_color(self, color: Color) -> Self;
    fn label_rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self;
    fn label_font_size(self, size: FontSize) -> Self;
    fn small_font(self) -> Self;
    fn medium_font(self) -> Self;
    fn large_font(self) -> Self;
}






/// A context on which the builder pattern can be implemented.
pub struct LabelContext<'a> {
    uic: &'a mut UiContext,
    text: &'a str,
    pos: Point,
    size: FontSize,
    maybe_color: Option<Color>,
}

impl<'a> LabelContext<'a> {
    /// A builder method for specifying font_size.
    pub fn size(self, size: FontSize) -> LabelContext<'a> {
        LabelContext { size: size, ..self }
    }
}

pub trait LabelBuilder<'a> {
    /// A label builder method to be implemented on the UiContext.
    fn label(&'a mut self, text: &'a str) -> LabelContext<'a>;
}

impl<'a> LabelBuilder<'a> for UiContext {

    /// A label builder method to be implemented on the UiContext.
    fn label(&'a mut self, text: &'a str) -> LabelContext<'a> {
        LabelContext {
            uic: self,
            text: text,
            pos: [0.0, 0.0],
            size: 24u32,
            maybe_color: None,
        }
    }

}

impl_colorable!(LabelContext);
impl_positionable!(LabelContext);

impl<'a> ::draw::Drawable for LabelContext<'a> {
    fn draw(&mut self, graphics: &mut Gl) {
        let color = self.maybe_color.unwrap_or(Color::black());
        self.uic.draw_text(graphics, self.pos, self.size, color, self.text);
    }
}
