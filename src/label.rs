
use color::Color;
use piston::{
    RenderArgs,
};
use opengl_graphics::{
    Gl,
    Texture,
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
use freetype;

type FontSize = u32;

/// An enum for passing in label information to widget arguments.
pub enum IsLabel<'a> {
    NoLabel,
    Label(&'a str, FontSize, Color),
}

/// Draw a label using the freetype font rendering backend.
pub fn draw(args: &RenderArgs,
            gl: &mut Gl,
            uic: &mut UIContext,
            pos: Point<f64>,
            size: FontSize,
            color: Color,
            text: &str) {
    let context = Context::abs(args.width as f64, args.height as f64)
                    .trans(pos.x, pos.y + size as f64);
    let face = &mut uic.face;
    face.set_pixel_sizes(0, size);
    let mut x = 0;
    let mut y = 0;
    let (r, g, b, a) = color.as_tuple();
    for ch in text.chars() {
        face.load_char(ch as u64, freetype::face::Render).unwrap();
        let glyph = face.glyph();
        let texture = Texture::from_memory_alpha(glyph.bitmap().buffer(),
                                                 glyph.bitmap().width() as u32,
                                                 glyph.bitmap().rows() as u32).unwrap();
        context.trans((x + glyph.bitmap_left()) as f64,
                      (y - glyph.bitmap_top()) as f64)
                        .image(&texture)
                        .rgba(r, g, b, a)
                        .draw(gl);
        x += (glyph.advance().x >> 6) as i32;
        y += (glyph.advance().y >> 6) as i32;
    }
}

/// Determine the pixel width of the final text bitmap.
pub fn get_text_width(uic: &mut UIContext, size: FontSize, text: &str) -> f64 {
    let face = &mut uic.face;
    face.set_pixel_sizes(0, size);
    let mut width = 0u32;
    for ch in text.chars() {
        face.load_char(ch as u64, freetype::face::Render).unwrap();
        let glyph = face.glyph();
        width += glyph.bitmap().width() as u32;
    }
    width as f64
}

