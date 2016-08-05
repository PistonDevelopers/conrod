//! Helper functions for using conrod with the `piston_window` crate.
//!
//! the `piston_window` crate attempts to provide a simple API over the gfx graphics backend and
//! the glutin window context and events.

extern crate piston_window;

use event;
use image;
use self::piston_window::{G2dTexture, PistonWindow};
use render;
use text;


/// A wrapper around a `G2dTexture` and a rusttype GPU `Cache`
///
/// Using a wrapper simplifies the construction of both caches and ensures that they maintain
/// identical dimensions.
pub struct GlyphCache {
    cache: text::GlyphCache,
    texture: G2dTexture<'static>,
    vertex_data: Vec<u8>,
}


impl GlyphCache {

    /// Constructor for a new `GlyphCache`.
    ///
    /// The `PistonWindow` provides the `Factory` argument to the `G2dTexture` constructor.
    ///
    /// The `width` and `height` arguments are in pixel values.
    ///
    /// If you need to resize the `GlyphCache`, construct a new one and discard the old one.
    pub fn new<B>(window: &mut PistonWindow<B>, width: u32, height: u32) -> GlyphCache 
        where B: self::piston_window::Window {

        // Construct the rusttype GPU cache with the tolerances recommended by their documentation.
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        let cache = text::GlyphCache::new(width, height, SCALE_TOLERANCE, POSITION_TOLERANCE);

        // Construct a `G2dTexture`
        let buffer_len = width as usize * height as usize;
        let init = vec![128; buffer_len];
        let settings = self::piston_window::TextureSettings::new();
        let factory = &mut window.factory;
        let texture = G2dTexture::from_memory_alpha(factory, &init, width, height, &settings).unwrap();

        GlyphCache {
            cache: cache,
            texture: texture,
            vertex_data: Vec::new(),
        }
    }

}


/// Converts any `GenericEvent` to a `Raw` conrod event.
pub fn convert_event<E, B>(event: E, window: &PistonWindow<B>) -> Option<event::Raw>
    where E: super::piston::event::GenericEvent,
          B: self::piston_window::Window,
{
    use self::piston_window::Window;
    use Scalar;

    let size = window.size();
    let (w, h) = (size.width as Scalar, size.height as Scalar);
    super::piston::event::convert(event, w, h)
}


/// Renders the given sequence of conrod primitives.
pub fn draw<'a, 'b, P, Img, F>(context: super::piston::draw::Context,
                               graphics: &'a mut piston_window::G2d<'b>,
                               primitives: P,
                               glyph_cache: &'a mut GlyphCache,
                               image_map: &'a image::Map<Img>,
                               texture_from_image: F)
    where P: render::PrimitiveWalker,
          F: FnMut(&Img) -> &G2dTexture<'static>,
{
    let GlyphCache { ref mut texture, ref mut cache, ref mut vertex_data } = *glyph_cache;

    // A function used for caching glyphs from `Text` widgets.
    let cache_queued_glyphs_fn = |graphics: &mut piston_window::G2d,
                                  cache: &mut G2dTexture<'static>,
                                  rect: text::rt::Rect<u32>,
                                  data: &[u8]|
    {
        cache_queued_glyphs(graphics, cache, rect, data, vertex_data);
    };

    super::piston::draw::primitives(
        primitives,
        context,
        graphics,
        texture,
        cache,
        image_map,
        cache_queued_glyphs_fn,
        texture_from_image,
    );
}

/// Draw a single `Primitive` to the screen.
///
/// This is useful if the user requires rendering primitives individually, perhaps to perform their
/// own rendering in between, etc.
pub fn draw_primitive<'a, 'b, Img, F>(context: super::piston::draw::Context,
                                      graphics: &'a mut piston_window::G2d<'b>,
                                      primitive: render::Primitive,
                                      glyph_cache: &'a mut GlyphCache,
                                      image_map: &'a image::Map<Img>,
                                      glyph_rectangles: &mut Vec<([f64; 4], [i32; 4])>,
                                      texture_from_image: F)
    where F: FnMut(&Img) -> &G2dTexture<'static>,
{
    let GlyphCache { ref mut texture, ref mut cache, ref mut vertex_data } = *glyph_cache;

    // A function used for caching glyphs from `Text` widgets.
    let cache_queued_glyphs_fn = |graphics: &mut piston_window::G2d,
                                  cache: &mut G2dTexture<'static>,
                                  rect: text::rt::Rect<u32>,
                                  data: &[u8]|
    {
        cache_queued_glyphs(graphics, cache, rect, data, vertex_data);
    };

    super::piston::draw::primitive(
        primitive,
        context,
        graphics,
        texture,
        cache,
        image_map,
        glyph_rectangles,
        cache_queued_glyphs_fn,
        texture_from_image,
    );
}

fn cache_queued_glyphs(graphics: &mut piston_window::G2d,
                       cache: &mut G2dTexture<'static>,
                       rect: text::rt::Rect<u32>,
                       data: &[u8],
                       vertex_data: &mut Vec<u8>)
{
    use self::piston_window::texture::UpdateTexture;

    // An iterator that efficiently maps the `byte`s yielded from `data` to `[r, g, b, byte]`;
    //
    // This is only used within the `cache_queued_glyphs` below, however due to a bug in rustc we
    // are unable to declare types inside the closure scope.
    struct Bytes { b: u8, i: u8 }
    impl Iterator for Bytes {
        type Item = u8;
        fn next(&mut self) -> Option<Self::Item> {
            let b = match self.i {
                0 => 255,
                1 => 255,
                2 => 255,
                3 => self.b,
                _ => return None,
            };
            self.i += 1;
            Some(b)
        }
    }

    let offset = [rect.min.x, rect.min.y];
    let size = [rect.width(), rect.height()];
    let format = self::piston_window::texture::Format::Rgba8;
    let encoder = &mut graphics.encoder;

    vertex_data.clear();
    vertex_data.extend(data.iter().flat_map(|&b| Bytes { b: b, i: 0 }));
    UpdateTexture::update(cache, encoder, format, &vertex_data[..], offset, size)
        .expect("Failed to update texture");
}
