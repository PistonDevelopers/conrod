//! A glium backend for rendering conrod primitives.

extern crate glium;

use render;

/// Render the given sequence of conrod primitive widgets.
pub fn primitives(mut primitives: render::Primitives) {
    while let Some(render::Primitive { kind, scizzor, rect }) = primitives.next() {
        match kind {

            render::PrimitiveKind::Rectangle { color } => {
            },

            render::PrimitiveKind::Polygon { color, points } => {
            },

            render::PrimitiveKind::Lines { color, cap, thickness, points } => {
            },

            render::PrimitiveKind::Text { color, glyph_cache, positioned_glyphs, font_id } => {

                // Cache the glyphs on the GPU.
                glyph_cache.cache_queued(|rect, data| {
                });

                // Render glyphs via the texture cache.
                let cache_id = font_id.index();
                for g in positioned_glyphs {
                    if let Ok(Some((uv_rect, screen_rect))) = glyph_cache.rect_for(cache_id, g) {
                    }
                }

            },

            render::PrimitiveKind::Image { maybe_color, texture_id, source_rect } => {
            },

            render::PrimitiveKind::Other(_) => (),
        }
    }
}
