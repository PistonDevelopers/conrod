
use color::Color;
use graphics::{
    Context,
    AddColor,
    AddImage,
    BackEnd,
    Draw,
    ImageSize,
    RelativeTransform2d,
};
use point::Point;
use ui_context::UiContext;

pub type FontSize = u32;

/// An enum for passing in label information to widget arguments.
pub enum Labeling<'a> {
    Label(&'a str, FontSize, Color),
    NoLabel,
}

/// Draw a label using the freetype font rendering backend.
pub fn draw<B: BackEnd<T>, T: ImageSize>(
    graphics: &mut B,
    uic: &mut UiContext<T>,
    pos: Point,
    size: FontSize,
    color: Color,
    text: &str
) {
    let mut x = 0;
    let mut y = 0;
    let (r, g, b, a) = color.as_tuple();
    let context = Context::abs(uic.win_w, uic.win_h)
                    .trans(pos[0], pos[1] + size as f64);
    for ch in text.chars() {
        let character = uic.get_character(size, ch);
        context.trans((x + character.bitmap_glyph.left()) as f64,
                      (y - character.bitmap_glyph.top()) as f64)
                        .image(&character.texture)
                        .rgba(r, g, b, a)
                        .draw(graphics);
        x += (character.glyph.advance().x >> 16) as i32;
        y += (character.glyph.advance().y >> 16) as i32;
    }
}

/// Determine the pixel width of the final text bitmap.
#[inline]
pub fn width<T>(uic: &mut UiContext<T>, size: FontSize, text: &str) -> f64 {
    text.chars().fold(0u32, |a, ch| {
        let character = uic.get_character(size, ch);
        a + (character.glyph.advance().x >> 16) as u32
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
pub struct LabelContext<'a, T: 'a> {
    uic: &'a mut UiContext<T>,
    text: &'a str,
    pos: Point,
    size: FontSize,
    maybe_color: Option<Color>,
}

impl<'a, T> LabelContext<'a, T> {
    /// A builder method for specifying font_size.
    pub fn size(self, size: FontSize) -> LabelContext<'a, T> {
        LabelContext { size: size, ..self }
    }
}

pub trait LabelBuilder<'a, T> {
    /// A label builder method to be implemented on the UiContext.
    fn label(&'a mut self, text: &'a str) -> LabelContext<'a, T>;
}

impl<'a, T> LabelBuilder<'a, T> for UiContext<T> {

    /// A label builder method to be implemented on the UiContext.
    fn label(&'a mut self, text: &'a str) -> LabelContext<'a, T> {
        LabelContext {
            uic: self,
            text: text,
            pos: [0.0, 0.0],
            size: 24u32,
            maybe_color: None,
        }
    }

}

impl_colorable!(LabelContext, T)
impl_positionable!(LabelContext, T)

impl<'a, T: ImageSize> ::draw::Drawable<T> for LabelContext<'a, T> {
    fn draw<B: BackEnd<T>>(&mut self, graphics: &mut B) {
        let color = self.maybe_color.unwrap_or(Color::black());
        draw(graphics, self.uic, self.pos, self.size, color, self.text);
    }
}

