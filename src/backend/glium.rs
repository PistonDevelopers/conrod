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

            render::PrimitiveKind::Text { color, positioned_glyphs, font_id } => {
            },

            render::PrimitiveKind::Image { maybe_color, texture_id, source_rect } => {
            },

            render::PrimitiveKind::Other(_) => (),
        }
    }
}
