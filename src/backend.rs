
use ::{CharacterCache, Point, Rect, Scalar};
use graph::{Container, Graph, Visitable};
use graphics::{self, Context, Graphics};
use std::iter::once;
use theme::Theme;
use widget::primitive;


/// Draw the given graph using the given graphics backend.
pub fn draw_from_graph<C, G>(character_cache: &mut C,
                             context: Context,
                             graphics: &mut G,
                             graph: &Graph,
                             depth_order: &[Visitable],
                             theme: &Theme)
    where C: CharacterCache,
          G: Graphics<Texture=C::Texture>,
{

    // A stack of contexts, one for each scroll group.
    let mut scroll_stack: Vec<Context> = Vec::new();

    // The depth order describes the order in which widgets should be drawn.
    for &visitable in depth_order {
        match visitable {

            Visitable::Widget(idx) => {
                if let Some(ref container) = graph.widget(idx) {

                    // Check the stack for the current Context.
                    let context = *scroll_stack.last().unwrap_or(&context);

                    // Draw the widget from its container.
                    draw_from_container(character_cache, &context, graphics, container, theme);

                    // If the current widget is some scrollable widget, we need to add a context
                    // for it to the top of the stack.
                    if let Some(scrolling) = container.maybe_scrolling {
                        let context = crop_context(context, scrolling.visible);
                        scroll_stack.push(context);
                    }
                }
            },

            Visitable::Scrollbar(idx) => {
                if let Some(scrolling) = graph.widget_scroll_state(idx) {

                    // Now that we've come across a scrollbar, we'll pop its Context from the
                    // scroll_stack and draw it if necessary.
                    scroll_stack.pop();

                    // TODO: Draw the scrollbar!
                }
            }

        }
    }
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
    // can't represent the negative coords with `u16` (the target DrawState dimension type).
    // We'll hold onto the lost negative values (x_neg and y_neg) so that we can compensate
    // with the width and height.
    let x_neg = if x < 0 { x } else { 0 };
    let y_neg = if y < 0 { y } else { 0 };
    let mut x = ::std::cmp::max(0, x) as u16;
    let mut y = ::std::cmp::max(0, y) as u16;
    let mut w = ::std::cmp::max(0, (w as i32 + x_neg)) as u16;
    let mut h = ::std::cmp::max(0, (h as i32 + y_neg)) as u16;

    // If there was already some scissor set, we must check for the intersection.
    if let Some(rect) = draw_state.scissor {
        if x + w < rect.x || rect.x + rect.w < x || y + h < rect.y || rect.y + rect.h < y {
            // If there is no intersection, we have no scissor.
            w = 0;
            h = 0;
        } else {
            // If there is some intersection, calculate the overlapping rect.
            let (a_l, a_r, a_b, a_t) = (x, x+w, y, y+h);
            let (b_l, b_r, b_b, b_t) = (rect.x, rect.x+rect.w, rect.y, rect.y+rect.h);
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

    Context { draw_state: draw_state.scissor(x, y, w, h), ..context }
}



/// Use the given graphics backend to draw the given widget.
pub fn draw_from_container<C, G>(character_cache: &mut C,
                                 context: &Context,
                                 graphics: &mut G,
                                 container: &Container,
                                 theme: &Theme)
    where C: CharacterCache,
          G: Graphics<Texture=C::Texture>,
{
    use widget::primitive::shape::Style as ShapeStyle;

    match container.kind {

        primitive::shape::rectangle::KIND => {
            if let Some(rectangle) = container.unique_widget_state::<::Rectangle>() {
                match rectangle.style {
                    ShapeStyle::Fill(_) => {
                        let color = rectangle.style.get_color(theme).to_fsa();
                        let (l, b, w, h) = container.rect.l_b_w_h();
                        let lbwh = [l, b, w, h];
                        let rectangle = graphics::Rectangle::new(color);
                        rectangle.draw(lbwh, &context.draw_state, context.transform, graphics);
                    },
                    ShapeStyle::Outline(line_style) => {
                        let (l, r, b, t) = container.rect.l_r_b_t();
                        let points = [[l, b], [l, t], [r, t], [r, b], [l, b]];
                        let points = points.iter().cloned();
                        draw_lines(context, graphics, theme, points, line_style);
                    },
                }
            }
        },

        primitive::shape::framed_rectangle::KIND => {
            if let Some(framed_rectangle) = container.unique_widget_state::<::FramedRectangle>() {
                let frame = framed_rectangle.style.get_frame(theme);
                if frame > 0.0 {
                    let frame_color = framed_rectangle.style.get_frame_color(theme).to_fsa();
                    let (l, b, w, h) = container.rect.l_b_w_h();
                    let lbwh = [l, b, w, h];
                    let rectangle = graphics::Rectangle::new(frame_color);
                    rectangle.draw(lbwh, &context.draw_state, context.transform, graphics);
                }
                let color = framed_rectangle.style.get_color(theme).to_fsa();
                let (l, b, w, h) = container.rect.l_b_w_h();
                let lbwh = [l, b, w, h];
                let rectangle = graphics::Rectangle::new(color);
                rectangle.draw(lbwh, &context.draw_state, context.transform, graphics);
            }
        },

        primitive::shape::oval::KIND => {
            if let Some(oval) = container.unique_widget_state::<::Oval>() {
                use std::f64::consts::PI;
                const CIRCLE_RESOLUTION: usize = 50;
                const NUM_POINTS: usize = CIRCLE_RESOLUTION + 1;
                let (x, y, w, h) = container.rect.x_y_w_h();
                let t = 2.0 * PI / CIRCLE_RESOLUTION as Scalar;
                let hw = w / 2.0;
                let hh = h / 2.0;
                let f = |i: Scalar| [x + hw * (t*i).cos(), y + hh * (t*i).sin()];
                let mut points = [[0.0, 0.0]; NUM_POINTS];
                for i in 0..NUM_POINTS {
                    points[i] = f(i as f64);
                }

                match oval.style {
                    ShapeStyle::Fill(_) => {
                        let color = oval.style.get_color(theme).to_fsa();
                        let polygon = graphics::Polygon::new(color);
                        polygon.draw(&points, &context.draw_state, context.transform, graphics);
                    },
                    ShapeStyle::Outline(line_style) => {
                        let points = points.iter().cloned();
                        draw_lines(context, graphics, theme, points, line_style)
                    },
                }
            }
        },

        primitive::shape::polygon::KIND => {
            use widget::primitive::shape::Style;
            use widget::primitive::shape::polygon::State;

            if let Some(polygon) = container.state_and_style::<State, Style>() {
                match polygon.style {
                    ShapeStyle::Fill(_) => {
                        let color = polygon.style.get_color(theme).to_fsa();
                        let points = &polygon.state.points[..];
                        let polygon = graphics::Polygon::new(color);
                        polygon.draw(points, &context.draw_state, context.transform, graphics);
                    },
                    ShapeStyle::Outline(line_style) => {
                        let mut points = polygon.state.points.iter().cloned();
                        let first = points.next();
                        let points = first.into_iter().chain(points).chain(first);
                        draw_lines(context, graphics, theme, points, line_style);
                    },
                }
            }
        },

        primitive::line::KIND => {
            if let Some(line) = container.unique_widget_state::<::Line>() {
                let points = once(line.state.start).chain(once(line.state.end));
                draw_lines(context, graphics, theme, points, line.style);
            }
        },

        primitive::point_path::KIND => {
            use widget::primitive::point_path::{State, Style};

            if let Some(point_path) = container.state_and_style::<State, Style>() {
                use widget::primitive::point_path::StyleKind;
                match point_path.style.get_kind(theme) {
                    // Draw only the lines of the **PointPath**.
                    StyleKind::Lines(line_style) => {
                        let points = point_path.state.points.iter().cloned();
                        draw_lines(context, graphics, theme, points, line_style);
                    },
                    // Draw only the Points of the **PointPath**.
                    StyleKind::Points => unimplemented!(),
                    // Draw only the lines of the **PointPath**.
                    StyleKind::Both(_) => unimplemented!(),
                }
            }
        },

        primitive::label::KIND => unimplemented!(),

        _ => (),
    }
}


/// Draw a series of lines between the given points using the given style.
pub fn draw_lines<G, I>(context: &Context,
                        graphics: &mut G,
                        theme: &Theme,
                        mut points: I,
                        style: primitive::line::Style)
    where G: Graphics,
          I: Iterator<Item=Point>,
{
    use widget::primitive::line::{Cap, Pattern};

    if let Some(first) = points.next() {
        let pattern = style.get_pattern(theme);
        let color = style.get_color(theme).to_fsa();
        let thickness = style.get_thickness(theme);
        let cap = style.get_cap(theme);
        match pattern {
            Pattern::Solid => {
                let line = match cap {
                    Cap::Flat => graphics::Line::new(color, thickness / 2.0),
                    Cap::Round => graphics::Line::new_round(color, thickness / 2.0),
                };
                let mut start = first;
                for end in points {
                    let coords = [start[0], start[1], end[0], end[1]];
                    line.draw(coords, &context.draw_state, context.transform, graphics);
                    start = end;
                }
            },
            Pattern::Dashed => unimplemented!(),
            Pattern::Dotted => unimplemented!(),
        }
    }
}

