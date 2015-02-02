use piston::quack::{ Pair, Set, SetAt };
use graphics::BackEnd;
use graphics::character::CharacterCache;
use color::Color;
use point::Point;
use ui_context::UiContext;
use Position;

pub type FontSize = u32;

/// An enum for passing in label information to widget arguments.
pub enum Labeling<'a> {
    Label(&'a str, FontSize, Color),
    NoLabel,
}

/// Determine the pixel width of the final text bitmap.
#[inline]
pub fn width<C: CharacterCache>(uic: &mut UiContext<C>, size: FontSize, text: &str) -> f64 {
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
    fn small_font<C>(self, uic: &UiContext<C>) -> Self;
    fn medium_font<C>(self, uic: &UiContext<C>) -> Self;
    fn large_font<C>(self, uic: &UiContext<C>) -> Self;
}

/// Label text property.
#[derive(Copy)]
pub struct LabelText<'a>(pub &'a str);

/// Label color property.
#[derive(Copy)]
pub struct LabelColor(pub Color);

/// Label font size property.
#[derive(Copy)]
pub struct LabelFontSize(pub FontSize);

impl<'a, T: 'a> Labelable<'a> for T
    where
        (LabelText<'a>, T): Pair<Data = LabelText<'a>, Object = T> + SetAt,
        (LabelColor, T): Pair<Data = LabelColor, Object = T> + SetAt,
        (LabelFontSize, T): Pair<Data = LabelFontSize, Object = T> + SetAt
{
    fn label(self, text: &'a str) -> Self {
        self.set(LabelText(text))
    }

    fn label_color(self, color: Color) -> Self {
        self.set(LabelColor(color))
    }

    fn label_rgba(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.set(LabelColor(Color([r, g, b, a])))
    }

    fn label_font_size(self, size: FontSize) -> Self {
        self.set(LabelFontSize(size))
    }

    fn small_font<C>(self, uic: &UiContext<C>) -> Self {
        self.set(LabelFontSize(uic.theme.font_size_small))
    }

    fn medium_font<C>(self, uic: &UiContext<C>) -> Self {
        self.set(LabelFontSize(uic.theme.font_size_medium))
    }

    fn large_font<C>(self, uic: &UiContext<C>) -> Self {
        self.set(LabelFontSize(uic.theme.font_size_large))
    }
}


/// A context on which the builder pattern can be implemented.
pub struct Label<'a> {
    text: &'a str,
    pos: Point,
    size: FontSize,
    maybe_color: Option<Color>,
}

impl<'a> Label<'a> {
    /// A builder method for specifying font_size.
    pub fn size(self, size: FontSize) -> Label<'a> {
        Label { size: size, ..self }
    }
}

impl<'a> Label<'a> {

    /// A label builder method to be implemented on the UiContext.
    pub fn new(text: &'a str) -> Label<'a> {
        Label {
            text: text,
            pos: [0.0, 0.0],
            size: 24u32,
            maybe_color: None,
        }
    }

}

quack! {
    label: Label['a]
    get:
    set:
        fn (val: Color) [] { label.maybe_color = Some(val) }
        fn (val: Position) [] { label.pos = val.0 }
    action:
}

impl<'a> ::draw::Drawable for Label<'a> {
    fn draw<B, C>(&mut self, uic: &mut UiContext<C>, graphics: &mut B)
        where
            B: BackEnd<Texture = <C as CharacterCache>::Texture>,
            C: CharacterCache
    {
        let color = self.maybe_color.unwrap_or(Color::black());
        uic.draw_text(graphics, self.pos, self.size, color, self.text);
    }
}
