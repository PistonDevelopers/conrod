
use color::Color;
use piston::{
    RenderArgs,
};
use opengl_graphics::{
    Gl,
};
use graphics::{
    Context,
    AddColor,
    AddImage,
    Draw,
    RelativeTransform2d,
};
use ui_context::UIContext;
use point::Point;

pub type FontSize = u32;

/// An enum for passing in label information to widget arguments.
pub enum Labeling<'a> {
    Label(&'a str, FontSize, Color),
    NoLabel,
}

/// Draw a label using the freetype font rendering backend.
pub fn draw(args: &RenderArgs,
            gl: &mut Gl,
            uic: &mut UIContext,
            pos: Point<f64>,
            size: FontSize,
            color: Color,
            text: &str) {
    let mut x = 0;
    let mut y = 0;
    let (r, g, b, a) = color.as_tuple();
    let context = Context::abs(args.width as f64, args.height as f64)
                    .trans(pos.x, pos.y + size as f64);
    for ch in text.chars() {
        let character = uic.get_character(size, ch);
        context.trans((x + character.bitmap_glyph.left()) as f64,
                      (y - character.bitmap_glyph.top()) as f64)
                        .image(&character.texture)
                        .rgba(r, g, b, a)
                        .draw(gl);
        x += (character.glyph.advance().x >> 16) as i32;
        y += (character.glyph.advance().y >> 16) as i32;
    }
}

/// Determine the pixel width of the final text bitmap.
#[inline]
pub fn width(uic: &mut UIContext, size: FontSize, text: &str) -> f64 {
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

