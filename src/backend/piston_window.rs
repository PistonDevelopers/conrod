//! Helper functions for using conrod with the `piston_window` crate.
//!
//! the `piston_window` crate attempts to provide a simple API over the gfx graphics backend and
//! the glutin window context and events.

extern crate piston_window;

use self::piston_window::{G2dTexture, PistonWindow};
use render;
use text;
use texture;

/// Constructor for a new texture cache.
pub fn new_text_texture_cache(window: &mut PistonWindow, w: u32, h: u32) -> G2dTexture<'static> {
    let buffer_len = w as usize * h as usize;
    let init = vec![128; buffer_len];
    let settings = self::piston_window::TextureSettings::new();
    G2dTexture::from_memory_alpha(&mut window.factory, &init, w, h, &settings).unwrap()
}

/// Renders the given sequence of conrod primitives.
pub fn draw<'a, 'b, F>(context: super::piston::draw::Context,
                       graphics: &'a mut piston_window::G2d<'b>,
                       primitives: render::Primitives,
                       text_texture_cache: &'a mut G2dTexture<'static>,
                       get_texture: F)
    where F: FnMut(texture::Id) -> Option<&'a G2dTexture<'static>>
{

    // A function used for caching glyphs from `Text` widgets.
    fn cache_queued_glyphs(graphics: &mut piston_window::G2d,
                           cache: &mut G2dTexture<'static>,
                           rect: text::rt::Rect<u32>,
                           data: &[u8])
    {
        use self::piston_window::texture::UpdateTexture;
        let offset = [rect.min.x, rect.min.y];
        let size = [rect.width(), rect.height()];
        let format = self::piston_window::texture::Format::Rgba8;
        let encoder = &mut graphics.encoder;

        // FIXME: These allocations and iterations are slow and messy.
        let new_data: Vec<_> = data.iter().flat_map(|&b| vec![255, 255, 255, b]).collect();
        UpdateTexture::update(cache, encoder, format, &new_data[..], offset, size)
            .expect("Failed to update texture");
    }

    // Data and functions for rendering the primitives.
    let renderer = super::piston::draw::Renderer {
        context: context,
        graphics: graphics,
        texture_cache: text_texture_cache,
        cache_queued_glyphs: cache_queued_glyphs,
        get_texture: get_texture,
    };

    super::piston::draw::primitives(primitives, renderer);
}
