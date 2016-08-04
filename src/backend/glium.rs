//! A glium backend for rendering conrod primitives.

extern crate glium;

use render;

/// Render the given sequence of conrod primitive widgets.
pub fn primitives(mut primitives: render::Primitives) {
    while let Some(render::Primitive { index, kind, scizzor, rect }) = primitives.next() {
        match kind {

            render::PrimitiveKind::Rectangle { color } => {
            },

            render::PrimitiveKind::Polygon { color, points } => {
            },

            render::PrimitiveKind::Lines { color, cap, thickness, points } => {
            },

            render::PrimitiveKind::Text { color, text, font_id } => {
            },

            render::PrimitiveKind::Image { color, source_rect } => {
            },

            render::PrimitiveKind::Other(_) => (),
        }
    }
}
