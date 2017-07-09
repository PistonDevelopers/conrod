//! Provides methods to draw conrod primitives.
//!
//! Primitives are drawn using the SDL renderer's capabilities or `SDL_gfx` for more involved
//! shapes.

use super::sdl2::render::{Canvas, RenderTarget, Texture};
use super::sdl2::rect::Rect as SdlRect;
use super::sdl2::pixels::Color as SdlColor;
use super::sdl2::gfx::primitives::DrawRenderer;

use render::{self, PrimitiveWalker};
use color::Color;
use position::rect::Rect;
use position::{Point, Scalar};
use text;
use image;

/// Renders a sequence of primitives.
///
/// # Parameters
///
/// * `primitives`: A `PrimitiveWalker` yielding the primitives to be rendered.
/// * `canvas`: An SDL `Canvas` the primitives are drawn onto.
/// * `text_texture_cache`: An SDL `Texture` acting as the on-GPU glyph cache.
/// * `glyph_cache`: The Rusttype glyph cache to use.
/// * `cache_queued_glyphs`: Callback function called when `text_texture_cache` should be updated
///   with new data.
/// * `image_map`: Map of image IDs to SDL `Texture`s.
pub fn primitives<'a, 't: 'a, P, U: 'a, C>(
    mut primitives: P,
    canvas: &'a mut Canvas<U>,
    text_texture_cache: &'a mut Texture<'t>,
    glyph_cache: &'a mut text::GlyphCache,
    mut cache_queued_glyphs: C,
    image_map: &'a mut image::Map<Texture<'t>>,
)
    where P: PrimitiveWalker,
          U: RenderTarget,
          C: FnMut(&mut Canvas<U>, &mut Texture, text::rt::Rect<u32>, &[u8]),
{
    let dpi_factor = dpi_factor();

    let viewport = canvas.viewport();
    let (voff_x, voff_y) = (viewport.w as Scalar / 2.0, viewport.h as Scalar / 2.0);

    // Maps a conrod point to SDL's coordinate system
    let map_point = |p: Point| {
        [(p[0] + voff_x) as i32, (- p[1] + voff_y) as i32]
    };

    // Translates a conrod `Rect` to SDL, changing coordinate systems.
    // SDL's coordinate system originates at the top left corner, y pointing downwards.
    let map_rect = |rect: Rect| SdlRect::new(
        map_point(rect.top_left())[0] as i32,
        map_point(rect.top_left())[1] as i32,
        rect.w() as u32,
        rect.h() as u32
    );

    let map_color = |color: Color| {
        let rgba = color.to_rgb();
        SdlColor {
            r: (rgba.0 * 255.0) as u8,
            g: (rgba.1 * 255.0) as u8,
            b: (rgba.2 * 255.0) as u8,
            a: (rgba.3 * 255.0) as u8,
        }
    };

    while let Some(primitive) = primitives.next_primitive() {
        let render::Primitive { kind, scizzor, rect, .. } = primitive;

        canvas.set_clip_rect(map_rect(scizzor));

        match kind {

            render::PrimitiveKind::Rectangle { color } => {
                canvas.set_draw_color(map_color(color));
                canvas.fill_rect(map_rect(rect));
            },

            render::PrimitiveKind::Polygon { color, points } => {
                let (mut vx, mut vy) = (Vec::new(), Vec::new());

                for p in points {
                    let p = map_point(*p);
                    vx.push(p[0] as i16);
                    vy.push(p[1] as i16);
                }

                canvas.filled_polygon(&vx, &vy, map_color(color)).unwrap();
            },

            render::PrimitiveKind::Lines { color, cap: _, thickness, points } => {
                // FIXME Doesn't handle the cap properly
                for points in points.windows(2) {
                    let (start, end) = (map_point(points[0]), map_point(points[1]));
                    canvas.thick_line(start[0] as i16, start[1] as i16, end[0] as i16, end[1] as i16, thickness as u8, map_color(color)).unwrap();
                }
            },

            render::PrimitiveKind::Text { color, text, font_id } => {

                let positioned_glyphs = text.positioned_glyphs(dpi_factor);

                // Queue the glyphs to be cached.
                for glyph in positioned_glyphs.iter() {
                    glyph_cache.queue_glyph(font_id.index(), glyph.clone());
                }

                // Cache the glyphs within the GPU cache.
                glyph_cache.cache_queued(|rect, data| {
                    cache_queued_glyphs(canvas, text_texture_cache, rect, data)
                }).unwrap();

                let cache_id = font_id.index();
                let query = text_texture_cache.query();
                let (tex_w, tex_h) = (query.width, query.height);

                let rectangles = positioned_glyphs.into_iter()
                    .filter_map(|g| glyph_cache.rect_for(cache_id, g).ok().unwrap_or(None))
                    .map(|(uv_rect, screen_rect)| {
                        let rectangle = {
                            let div_dpi_factor = |s| (s as f32 / dpi_factor as f32) as f64;
                            let left = div_dpi_factor(screen_rect.min.x);
                            let top = div_dpi_factor(screen_rect.min.y);
                            let right = div_dpi_factor(screen_rect.max.x);
                            let bottom = div_dpi_factor(screen_rect.max.y);
                            let w = right - left;
                            let h = bottom - top;
                            Some(SdlRect::new(left as i32, top as i32, w as u32, h as u32))
                        };
                        let source_rectangle = {
                            let x = (uv_rect.min.x * tex_w as f32) as f64;
                            let y = (uv_rect.min.y * tex_h as f32) as f64;
                            let w = ((uv_rect.max.x - uv_rect.min.x) * tex_w as f32) as f64;
                            let h = ((uv_rect.max.y - uv_rect.min.y) * tex_h as f32) as f64;
                            Some(SdlRect::new(x as i32, y as i32, w as u32, h as u32))
                        };
                        (rectangle, source_rectangle)
                    });

                let color = map_color(color);
                text_texture_cache.set_color_mod(color.r, color.g, color.b);
                text_texture_cache.set_alpha_mod(color.a);

                for (dest, src) in rectangles {
                    canvas.copy(&text_texture_cache, src, dest).unwrap();
                }
            },

            render::PrimitiveKind::Image { image_id, color, source_rect } => {
                if let Some(img) = image_map.get_mut(image_id) {
                    if let Some(color) = color {
                        let color = map_color(color);
                        img.set_color_mod(color.r, color.g, color.b);
                        img.set_alpha_mod(color.a);
                    }

                    canvas.copy(img, source_rect.map(|r| SdlRect::new(r.x.start as i32, r.y.start as i32, r.w() as u32, r.h() as u32)), map_rect(rect)).unwrap();
                } else {
                    panic!("no entry in image map for id {:?}", image_id);
                }
            },

            render::PrimitiveKind::Other(_widget) => {},

            _ => {},

        }
    }
}

fn dpi_factor() -> f32 {
    // FIXME: We can calculate the DPI factor by accessing the video subsystem
    1.0
}
