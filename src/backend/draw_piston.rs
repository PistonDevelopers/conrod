//! A piston backend for rendering conrod primitives.

use Rect;
use piston_graphics;
use render;

#[doc(inline)]
pub use piston_graphics::{Context, DrawState, Graphics, ImageSize, Transformed};


/// Render the given sequence of conrod primitive widgets.
pub fn primitives<G>(mut primitives: render::Primitives, context: Context, graphics: &mut G)
    where G: Graphics,
{
    // Translate the `context` to suit conrod's orientation (middle (0, 0), y pointint upwards).
    let view_size = context.get_view_size();
    let context = context.trans(view_size[0] / 2.0, view_size[1] / 2.0).scale(1.0, -1.0);

    primitives.draw(|render::Primitive { kind, scizzor, rect }| {
        let context = crop_context(context, scizzor);

        match kind {

            render::PrimitiveKind::Rectangle { color } => {
                let (l, b, w, h) = rect.l_b_w_h();
                let lbwh = [l, b, w, h];
                let rectangle = piston_graphics::Rectangle::new(color.to_fsa());
                rectangle.draw(lbwh, &context.draw_state, context.transform, graphics);
            },

            render::PrimitiveKind::Polygon { color, points } => {
                let color = color.to_fsa();
                let polygon = piston_graphics::Polygon::new(color);
                polygon.draw(points, &context.draw_state, context.transform, graphics);
            },

            render::PrimitiveKind::Lines { color, cap, thickness, points } => {
                use widget::primitive::line::Cap;
                let color = color.to_fsa();

                let mut points = points.iter();
                if let Some(first) = points.next() {
                    let line = match cap {
                        Cap::Flat => piston_graphics::Line::new(color, thickness / 2.0),
                        Cap::Round => piston_graphics::Line::new_round(color, thickness / 2.0),
                    };
                    let mut start = first;
                    for end in points {
                        let coords = [start[0], start[1], end[0], end[1]];
                        line.draw(coords, &context.draw_state, context.transform, graphics);
                        start = end;
                    }
                }
            },

            render::PrimitiveKind::Text {
                color, text, line_infos, font_size, line_spacing, x_align, y_align
            } => {
                use text;
                let line_infos = line_infos.iter().cloned();
                let lines = line_infos.clone().map(|info| &text[info.byte_range()]);
                let line_rects =
                    text::line::rects(line_infos, font_size, rect, x_align, y_align, line_spacing);
                for (line, line_rect) in lines.zip(line_rects) {
                    let offset = [line_rect.left().round(), line_rect.bottom().round()];
                    let context = context.trans(offset[0], offset[1]).scale(1.0, -1.0);
                    let transform = context.transform;
                    let draw_state = &context.draw_state;
                    // piston_graphics::text::Text::new_color(color, font_size)
                    //     .round()
                    //     .draw(line, character_cache, draw_state, transform, graphics);
                }
            },

            render::PrimitiveKind::Image { texture_index, source_rect } => {
                // if let Some(texture) = state.texture.as_ref() {
                //     let mut image = piston_graphics::image::Image::new();
                //     image.color = style.maybe_color.and_then(|c| c.map(|c| c.to_fsa()));
                //     image.source_rectangle = Some({
                //         let (x, y, w, h) = texture.src_rect.x_y_w_h();
                //         [x as i32, y as i32, w as i32, h as i32]
                //     });
                //     let (left, top, w, h) = rect.l_t_w_h();
                //     image.rectangle = Some([0.0, 0.0, w, h]);
                //     let context = context.trans(left, top).scale(1.0, -1.0);
                //     let transform = context.transform;
                //     let draw_state = &context.draw_state;
                //     image.draw(texture.arc.as_ref(), draw_state, transform, graphics);
                // }
            },

        }
    });
}


/// Crop the given **Context** to the given **Rect**.
///
/// This is non-trivial as we must consider the view_size, viewport, the difference in
/// co-ordinate systems and the conversion from `f64` dimensions to `u16`.
fn crop_context(context: Context, rect: Rect) -> Context {
    use utils::map_range;
    let Context { draw_state, .. } = context;

    let (x, y, w, h) = rect.x_y_w_h();

    // Our view_dim is our virtual window size which is consistent no matter the display.
    let view_dim = context.get_view_size();

    // Our draw_dim is the actual window size in pixels. Our target crop area must be
    // represented in this size.
    let draw_dim = match context.viewport {
        Some(viewport) => [viewport.draw_size[0] as f64, viewport.draw_size[1] as f64],
        None => view_dim,
    };

    // Calculate the distance to the edges of the window from the center.
    let left = -view_dim[0] / 2.0;
    let right = view_dim[0] / 2.0;
    let bottom = -view_dim[1] / 2.0;
    let top = view_dim[1] / 2.0;

    // We start with the x and y in the center of our crop area, however we need it to be
    // at the top left of the crop area.
    let left_x = x - w as f64 / 2.0;
    let top_y = y - h as f64 / 2.0;

    // Map the position at the top left of the crop area in view_dim to our draw_dim.
    let x = map_range(left_x, left, right, 0, draw_dim[0] as i32);
    let y = map_range(top_y, bottom, top, 0, draw_dim[1] as i32);

    // Convert the w and h from our view_dim to the draw_dim.
    let w_scale = draw_dim[0] / view_dim[0];
    let h_scale = draw_dim[1] / view_dim[1];
    let w = w * w_scale;
    let h = h * h_scale;

    // If we ended up with negative coords for the crop area, we'll use 0 instead as we
    // can't represent the negative coords with `u32` (the target DrawState dimension type).
    // We'll hold onto the lost negative values (x_neg and y_neg) so that we can compensate
    // with the width and height.
    let x_neg = if x < 0 { x } else { 0 };
    let y_neg = if y < 0 { y } else { 0 };
    let mut x = ::std::cmp::max(0, x) as u32;
    let mut y = ::std::cmp::max(0, y) as u32;
    let mut w = ::std::cmp::max(0, (w as i32 + x_neg)) as u32;
    let mut h = ::std::cmp::max(0, (h as i32 + y_neg)) as u32;

    // If there was already some scissor set, we must check for the intersection.
    if let Some(rect) = draw_state.scissor {
        let (r_x, r_y, r_w, r_h) = (rect[0], rect[1], rect[2], rect[3]);
        if x + w < r_x || r_x + r_w < x || y + h < r_y || r_y + r_h < y {
            // If there is no intersection, we have no scissor.
            w = 0;
            h = 0;
        } else {
            // If there is some intersection, calculate the overlapping rect.
            let (a_l, a_r, a_b, a_t) = (x, x+w, y, y+h);
            let (b_l, b_r, b_b, b_t) = (r_x, r_x+r_w, r_y, r_y+r_h);
            let l = if a_l > b_l { a_l } else { b_l };
            let r = if a_r < b_r { a_r } else { b_r };
            let b = if a_b > b_b { a_b } else { b_b };
            let t = if a_t < b_t { a_t } else { b_t };
            x = l;
            y = b;
            w = r - l;
            h = t - b;
        }
    }

    Context { draw_state: draw_state.scissor([x, y, w, h]), ..context }
}