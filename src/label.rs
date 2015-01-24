use quack::{ GetFrom, SetAt };
use graphics;
use color::Color;
use opengl_graphics::Gl;
use internal;
use ui_context::UiContext;
use Position;
use FontSize;
use MaybeColor;

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
#[derive(Copy)]
pub struct Label<'a> {
    text: &'a str,
    pos: internal::Point,
    size: internal::FontSize,
    maybe_color: Option<internal::Color>,
}

impl<'a> Label<'a> {
    /// Creates a new label.
    pub fn new(text: &'a str) -> Label<'a> {
        Label {
            text: text,
            pos: [0.0, 0.0],
            size: 24u32,
            maybe_color: None,
        }
    }

    pub fn draw(&self, uic: &mut UiContext, back_end: &mut Gl) {
        let color = self.maybe_color.unwrap_or(graphics::color::BLACK);
        uic.draw_text(back_end, self.pos, self.size, Color(color), self.text);
    }
}

impl<'a> GetFrom for (Position, Label<'a>) {
    type Property = Position;
    type Object = Label<'a>;

    fn get_from(label: &Label<'a>) -> Position {
        Position(label.pos)
    }
}

impl<'a> SetAt for (Position, Label<'a>) {
    type Property = Position;
    type Object = Label<'a>;

    fn set_at(Position(pos): Position, label: &mut Label<'a>) {
        label.pos = pos;
    }
}

impl<'a> GetFrom for (FontSize, Label<'a>) {
    type Property = FontSize;
    type Object = Label<'a>;

    fn get_from(label: &Label<'a>) -> FontSize {
        FontSize(label.size)
    }
}

impl<'a> SetAt for (FontSize, Label<'a>) {
    type Property = FontSize;
    type Object = Label<'a>;

    fn set_at(FontSize(size): FontSize, label: &mut Label<'a>) {
        label.size = size;
    }
}

impl<'a> GetFrom for (MaybeColor, Label<'a>) {
    type Property = MaybeColor;
    type Object = Label<'a>;

    fn get_from(label: &Label<'a>) -> MaybeColor {
        MaybeColor(label.maybe_color)
    }
}

impl<'a> SetAt for (MaybeColor, Label<'a>) {
    type Property = MaybeColor;
    type Object = Label<'a>;

    fn set_at(MaybeColor(maybe_color): MaybeColor, label: &mut Label<'a>) {
        label.maybe_color = maybe_color;
    }
}

impl<'a> SetAt for (Color, Label<'a>) {
    type Property = Color;
    type Object = Label<'a>;

    fn set_at(Color(color): Color, label: &mut Label<'a>) {
        label.maybe_color = Some(color);
    }
}

#[derive(Copy)]
pub struct Text<'a>(pub &'a str);

impl<'a> GetFrom for (Text<'a>, Label<'a>) {
    type Property = Text<'a>;
    type Object = Label<'a>;

    fn get_from(label: &Label<'a>) -> Text<'a> {
        Text(label.text)
    }
}

impl<'a> SetAt for (Text<'a>, Label<'a>) {
    type Property = Text<'a>;
    type Object = Label<'a>;

    fn set_at(Text(text): Text<'a>, label: &mut Label<'a>) {
        label.text = text;
    }
}
