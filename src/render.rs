//! Conrod's generic graphics backend.
//!
//! **Note:** Conrod currently uses Piston's generic [graphics
//! crate](https://github.com/PistonDevelopers/graphics) (and specifically the
//! [**Graphics**](http://docs.piston.rs/graphics/graphics/trait.Graphics.html))
//! trait to enable genericity over custom user backends. This dependency may change in the near
//! future in favour of a simplified conrod-specific graphics and character caching backend trait.
//!
//! This is the only module in which the piston graphics crate will be used directly.


use {Align, Color, Dimensions, Point, Rect, Scalar};
use graph::{self, Graph, NodeIndex};
use std;
use text;
use texture;
use theme::Theme;
use widget::primitive;


/// An iterator yielding a reference to each primitive in order of depth for rendering.
pub struct Primitives<'a> {
    crop_stack: Vec<(NodeIndex, Rect)>,
    depth_order: std::slice::Iter<'a, NodeIndex>,
    graph: &'a Graph,
    theme: &'a Theme,
    fonts: &'a text::font::Map,
    window_rect: Rect,
    /// The point slice to use for the `Lines` and `Polygon` primitives.
    points: Vec<Point>,
    /// The slice of rusttype `PositionedGlyph`s to re-use for the `Text` primitive.
    positioned_glyphs: Vec<text::PositionedGlyph>,
    /// The GPU cache for caching `Text` glyphs.
    glyph_cache: &'a mut text::GlyphCache,
}

/// Data required for rendering a single primitive widget.
pub struct Primitive<'a> {
    /// State and style for this primitive widget.
    pub kind: PrimitiveKind<'a>,
    /// The Rect to which the primitive widget should be cropped.
    ///
    /// Only parts of the widget within this `Rect` should be drawn.
    pub scizzor: Rect,
    /// The bounding rectangle for the `Primitive`.
    pub rect: Rect,
}

/// The unique kind for each primitive element in the Ui.
pub enum PrimitiveKind<'a> {

    /// A filled `Rectangle`.
    ///
    /// These are produced by the `Rectangle` and `FramedRectangle` primitive widgets. A `Filled`
    /// `Rectangle` widget produces a single `Rectangle`. The `FramedRectangle` produces two
    /// `Rectangle`s, the first for the outer frame and the second for the inner on top.
    Rectangle {
        /// The fill colour for the rectangle. 
        color: Color
    },

    /// A filled `Polygon`.
    ///
    /// These are produced by the `Oval` and `Polygon` primitive widgets.
    Polygon {
        /// The fill colour for the inner part of the polygon
        color: Color,
        /// The ordered points that, when joined with lines, represent each side of the `Polygon`.
        ///
        /// The first and final points should always be the same.
        points: &'a [Point],
    },

    /// A series of consecutive `Line`s.
    ///
    /// These are produces via the `Line` and `PointPath` primitive widgets, or the `shape`
    /// primitives if they are instantiated with an `Outline` style.
    Lines {
        /// The colour of each `Line`.
        color: Color,
        /// Whether the end of the lines should be `Flat` or `Round`.
        cap: primitive::line::Cap,
        /// The thickness of the lines, i.e. the width of a vertical line or th height of a
        /// horizontal line.
        thickness: Scalar,
        /// The ordered points which should be joined by lines.
        points: &'a [Point],
    },

    /// A single `Image`, produced by the primitive `Image` widget.
    Image {
        /// The unique identifier for the texture used by the `Image`.
        ///
        /// This is normally produced by a `conrod::texture::Map` instance.
        texture_id: texture::Id,
        /// When `Some`, colours the `Image`. When `None`, the `Image` uses its regular colours.
        maybe_color: Option<Color>,
        /// The area of the texture that will be drawn to the `Image`'s `Rect`.
        source_rect: Option<Rect>,
    },

    /// A single block of `Text`, produced by the primitive `Text` widget.
    Text {
        /// The colour of the `Text`.
        color: Color,
        /// The RustType GPU cache, used for caching `PositionedGlyph` within some texture in GPU
        /// memory.
        ///
        /// The `positioned_glyphs` will be queued for caching within the `glyph_cache` just prior
        /// to being passed to the user `draw_primitive` function.
        glyph_cache: &'a mut text::GlyphCache,
        /// All glyphs within the `Text` laid out in their correct positions in order from top-left
        /// to bottom right.
        positioned_glyphs: &'a [text::PositionedGlyph],
        /// The unique identifier for the font, used for the `glyph_cache.rect_for(id, glyph)`
        /// method.
        font_id: text::font::Id,
    },

    /// An `Other` variant will be yielded for every non-primitive widget in the list.
    ///
    /// Most of the time, this variant can be ignored, however it is useful for users who need to
    /// render widgets in ways that cannot be covered by the other `PrimitiveKind` variants.
    ///
    /// For example, a `Shader` widget might be required for updating uniforms in user rendering
    /// code. In order to access the unique state of this widget, the user can check `Other`
    /// variants for a container whose `kind` field matches the unique kind of the `Shader` widget.
    /// They can then retrieve the unique state of the widget and cast it to its actual type using
    /// either of the `Container::state_and_style` or `Container::unique_widget_state` methods.
    Other(&'a graph::Container),

}


const CIRCLE_RESOLUTION: usize = 50;
const NUM_POINTS: usize = CIRCLE_RESOLUTION + 1;


impl<'a> Primitives<'a> {

    /// Constructor for the `Primitives` iterator.
    pub fn new(graph: &'a Graph,
               depth_order: &'a [NodeIndex],
               theme: &'a Theme,
               fonts: &'a text::font::Map,
               glyph_cache: &'a mut text::GlyphCache,
               window_dim: Dimensions) -> Self
    {
        Primitives {
            crop_stack: Vec::new(),
            depth_order: depth_order.iter(),
            graph: graph,
            theme: theme,
            fonts: fonts,
            window_rect: Rect::from_xy_dim([0.0, 0.0], window_dim),
            // Initialise the `points` `Vec` with at least as many points as there are in an
            // outlined `Rectangle`. This saves us from having to check the length of the buffer
            // before writing points for an `Oval` or `Rectangle`.
            points: vec![[0.0, 0.0]; NUM_POINTS],
            positioned_glyphs: Vec::new(),
            glyph_cache: glyph_cache,
        }
    }

    /// Yield the next `Primitive` for rendering.
    pub fn next(&mut self) -> Option<Primitive> {
        let Primitives {
            ref mut crop_stack,
            ref mut depth_order,
            graph,
            theme,
            fonts,
            window_rect,
            ref mut points,
            ref mut positioned_glyphs,
            ref mut glyph_cache,
        } = *self;

        let (window_w, window_h) = window_rect.w_h();

        let trans_x = |x: Scalar| x + window_w / 2.0;
        let trans_y = |y: Scalar| (-y) + window_h / 2.0;

        while let Some(widget) = next_widget(depth_order, graph, crop_stack, window_rect) {
            use widget::primitive::shape::Style as ShapeStyle;
            let (scizzor, container) = widget;
            let rect = container.rect;

            // Extract the unique state and style from the container.
            match container.kind {

                primitive::shape::rectangle::KIND => {
                    if let Some(rectangle) = container.unique_widget_state::<::Rectangle>() {
                        let graph::UniqueWidgetState { ref style, .. } = *rectangle;
                        let color = style.get_color(theme);
                        match *style {
                            ShapeStyle::Fill(_) => {
                                let kind = PrimitiveKind::Rectangle { color: color };
                                return Some(new_primitive(kind, scizzor, rect));
                            },
                            ShapeStyle::Outline(ref line_style) => {
                                let (l, r, b, t) = rect.l_r_b_t();
                                points[0] = [l, b];
                                points[1] = [l, t];
                                points[2] = [r, t];
                                points[3] = [r, b];
                                points[4] = [l, b];
                                let cap = line_style.get_cap(theme);
                                let thickness = line_style.get_thickness(theme);
                                let kind = PrimitiveKind::Lines {
                                    color: color,
                                    cap: cap,
                                    thickness: thickness,
                                    points: &points[..5],
                                };
                                return Some(new_primitive(kind, scizzor, rect));
                            },
                        }
                    }
                },

                primitive::shape::oval::KIND => {
                    if let Some(oval) = container.unique_widget_state::<::Oval>() {
                        use std::f64::consts::PI;
                        let graph::UniqueWidgetState { ref style, .. } = *oval;

                        let (x, y, w, h) = rect.x_y_w_h();
                        let t = 2.0 * PI / CIRCLE_RESOLUTION as Scalar;
                        let hw = w / 2.0;
                        let hh = h / 2.0;
                        let f = |i: Scalar| [x + hw * (t*i).cos(), y + hh * (t*i).sin()];
                        for i in 0..NUM_POINTS {
                            points[i] = f(i as f64);
                        }

                        let color = style.get_color(theme);
                        let points = &mut points[..NUM_POINTS];
                        match *style {
                            ShapeStyle::Fill(_) => {
                                let kind = PrimitiveKind::Polygon { color: color, points: points };
                                return Some(new_primitive(kind, scizzor, rect));
                            },
                            ShapeStyle::Outline(ref line_style) => {
                                let cap = line_style.get_cap(theme);
                                let thickness = line_style.get_thickness(theme);
                                let kind = PrimitiveKind::Lines {
                                    color: color,
                                    cap: cap,
                                    thickness: thickness,
                                    points: points,
                                };
                                return Some(new_primitive(kind, scizzor, rect));
                            },
                        }
                    }
                },

                primitive::shape::polygon::KIND => {
                    use widget::primitive::shape::Style;
                    use widget::primitive::shape::polygon::State;
                    if let Some(polygon) = container.state_and_style::<State, Style>() {
                        let graph::UniqueWidgetState { ref state, ref style } = *polygon;

                        let color = style.get_color(theme);
                        let points = &state.points[..];
                        match *style {
                            ShapeStyle::Fill(_) => {
                                let kind = PrimitiveKind::Polygon { color: color, points: points };
                                return Some(new_primitive(kind, scizzor, rect));
                            },
                            ShapeStyle::Outline(ref line_style) => {
                                let cap = line_style.get_cap(theme);
                                let thickness = line_style.get_thickness(theme);
                                let kind = PrimitiveKind::Lines {
                                    color: color,
                                    cap: cap,
                                    thickness: thickness,
                                    points: points,
                                };
                                return Some(new_primitive(kind, scizzor, rect));
                            },
                        }
                    }
                },

                primitive::line::KIND => {
                    if let Some(line) = container.unique_widget_state::<::Line>() {
                        let graph::UniqueWidgetState { ref state, ref style } = *line;
                        let color = style.get_color(theme);
                        let cap = style.get_cap(theme);
                        let thickness = style.get_thickness(theme);
                        points[0] = state.start;
                        points[1] = state.end;
                        let points = &points[..2];
                        let kind = PrimitiveKind::Lines {
                            color: color,
                            cap: cap,
                            thickness: thickness,
                            points: points,
                        };
                        return Some(new_primitive(kind, scizzor, rect));
                    }
                },

                primitive::point_path::KIND => {
                    use widget::primitive::point_path::{State, Style};
                    if let Some(point_path) = container.state_and_style::<State, Style>() {
                        let graph::UniqueWidgetState { ref state, ref style } = *point_path;
                        let color = style.get_color(theme);
                        let cap = style.get_cap(theme);
                        let thickness = style.get_thickness(theme);
                        let points = &state.points[..];
                        let kind = PrimitiveKind::Lines {
                            color: color,
                            cap: cap,
                            thickness: thickness,
                            points: points,
                        };
                        return Some(new_primitive(kind, scizzor, rect));
                    }
                },

                primitive::text::KIND => {
                    if let Some(text) = container.unique_widget_state::<::Text>() {
                        let graph::UniqueWidgetState { ref state, ref style } = *text;
                        let font_id = match style.font_id(theme).or_else(|| fonts.ids().next()) {
                            Some(id) => id,
                            None => continue,
                        };
                        let font = match fonts.get(font_id) {
                            Some(font) => font,
                            None => continue,
                        };

                        // Retrieve styling.
                        let color = style.color(theme);
                        let font_size = style.font_size(theme);
                        let line_spacing = style.line_spacing(theme);
                        let x_align = style.text_align(theme);
                        let y_align = Align::End;
                        let scale = text::pt_to_scale(font_size);

                        // Produce the text layout iterators.
                        let line_infos = state.line_infos.iter().cloned();
                        let lines = line_infos.clone().map(|info| &state.string[info.byte_range()]);
                        let line_rects = text::line::rects(line_infos, font_size, rect,
                                                           x_align, y_align, line_spacing);

                        // Clear the existing glyphs and fill the buffer with glyphs for this Text.
                        positioned_glyphs.clear();
                        for (line, line_rect) in lines.zip(line_rects) {
                            let (x, y) = (trans_x(line_rect.left()) as f32, trans_y(line_rect.bottom()) as f32);
                            let point = text::rt::Point { x: x, y: y };
                            positioned_glyphs.extend(font.layout(line, scale, point).map(|g| g.standalone()));
                        }

                        // Queue the glyphs to be cached.
                        for glyph in positioned_glyphs.iter() {
                            glyph_cache.queue_glyph(font_id.index(), glyph.clone());
                        }

                        let kind = PrimitiveKind::Text {
                            color: color,
                            glyph_cache: glyph_cache,
                            positioned_glyphs: positioned_glyphs,
                            font_id: font_id,
                        };
                        return Some(new_primitive(kind, scizzor, rect));
                    }
                },

                primitive::image::KIND => {
                    use widget::primitive::image::{State, Style};
                    if let Some(image) = container.state_and_style::<State, Style>() {
                        let graph::UniqueWidgetState { ref state, ref style } = *image;
                        let maybe_color = style.maybe_color(theme);
                        let kind = PrimitiveKind::Image {
                            maybe_color: maybe_color,
                            texture_id: state.texture_id,
                            source_rect: state.src_rect,
                        };
                        return Some(new_primitive(kind, scizzor, rect));
                    }
                },

                // Return an `Other` variant for all non-primitive widgets.
                _ => {
                    let kind = PrimitiveKind::Other(container);
                    return Some(new_primitive(kind, scizzor, rect));
                },
            }
        }

        None
    }
}



/// Simplify the constructor for a `Primitive`.
fn new_primitive(kind: PrimitiveKind, scizzor: Rect, rect: Rect) -> Primitive {
    Primitive {
        kind: kind,
        scizzor: scizzor,
        rect: rect,
    }
}

/// Retrieves the next visible widget from the `depth_order`, updating the `crop_stack` as
/// necessary.
fn next_widget<'a>(depth_order: &mut std::slice::Iter<NodeIndex>,
                   graph: &'a Graph,
                   crop_stack: &mut Vec<(NodeIndex, Rect)>,
                   window_rect: Rect) -> Option<(Rect, &'a graph::Container)>
{
    while let Some(&node_index) = depth_order.next() {
        let container = match graph.widget(node_index) {
            Some(container) => container,
            None => continue,
        };

        // If we're currently using a cropped context and the current `crop_parent_idx` is
        // *not* a depth-wise parent of the widget at the current `idx`, we should pop that
        // cropped context from the stack as we are done with it.
        while let Some(&(crop_parent_idx, _)) = crop_stack.last() {
            if graph.does_recursive_depth_edge_exist(crop_parent_idx, node_index) {
                break;
            } else {
                crop_stack.pop();
            }
        }

        // Check the stack for the current Context.
        let scizzor = crop_stack.last().map(|&(_, scizzor)| scizzor).unwrap_or(window_rect);

        // If the current widget should crop its children, we need to add a rect for it to
        // the top of the crop stack.
        if container.crop_kids {
            let scizzor_rect = container.kid_area.rect.overlap(scizzor)
                .unwrap_or_else(|| Rect::from_xy_dim([0.0, 0.0], [0.0, 0.0]));
            crop_stack.push((node_index, scizzor_rect));
        }

        // We only want to return primitives that are actually visible.
        let is_visible = container.rect.overlap(window_rect).is_some()
            && graph::algo::cropped_area_of_widget(graph, node_index).is_some();
        if !is_visible {
            continue;
        }

        return Some((scizzor, container));
    }

    None
}


// impl<'a> Primitives<'a> {
// 
//     /// Call the given `draw_primitivese` function with all `Primitive`s that make up the `Ui` in
//     /// its current state.
//     ///
//     /// `Primitive`s are yielded in order of depth, bottom to top.
//     pub fn draw<F>(&mut self, mut draw_primitive: F)
//         where F: FnMut(Primitive),
//     {
//         let Primitives {
//             ref mut crop_stack,
//             ref mut depth_order,
//             graph,
//             theme,
//             fonts,
//             window_rect,
//             ref mut points,
//             ref mut positioned_glyphs,
//             ref mut glyph_cache,
//         } = *self;
// 
//         while let Some(&node_index) = depth_order.next() {
//             use widget::primitive::shape::Style as ShapeStyle;
// 
//             let container = match graph.widget(node_index) {
//                 Some(container) => container,
//                 None => continue,
//             };
// 
//             // If we're currently using a cropped context and the current `crop_parent_idx` is
//             // *not* a depth-wise parent of the widget at the current `idx`, we should pop that
//             // cropped context from the stack as we are done with it.
//             while let Some(&(crop_parent_idx, _)) = crop_stack.last() {
//                 if graph.does_recursive_depth_edge_exist(crop_parent_idx, node_index) {
//                     break;
//                 } else {
//                     crop_stack.pop();
//                 }
//             }
// 
//             // Check the stack for the current Context.
//             let scizzor = crop_stack.last().map(|&(_, scizzor)| scizzor).unwrap_or(window_rect);
// 
//             // If the current widget should crop its children, we need to add a rect for it to
//             // the top of the crop stack.
//             if container.crop_kids {
//                 let scizzor_rect = container.kid_area.rect.overlap(scizzor)
//                     .unwrap_or_else(|| Rect::from_xy_dim([0.0, 0.0], [0.0, 0.0]));
//                 crop_stack.push((node_index, scizzor_rect));
//             }
// 
//             // We only want to return primitives that are actually visible.
//             let is_visible = container.rect.overlap(window_rect).is_some()
//                 && graph::algo::cropped_area_of_widget(graph, node_index).is_some();
//             if !is_visible {
//                 continue;
//             }
// 
//             let rect = container.rect;
// 
//             // Simplify the constructor for a `Primitive`.
//             fn new_primitive(kind: PrimitiveKind, scizzor: Rect, rect: Rect) -> Primitive {
//                 Primitive {
//                     kind: kind,
//                     scizzor: scizzor,
//                     rect: rect,
//                 }
//             }
// 
//             // Extract the unique state and style from the container.
//             match container.kind {
// 
//                 primitive::shape::rectangle::KIND => {
//                     if let Some(rectangle) = container.unique_widget_state::<::Rectangle>() {
//                         let graph::UniqueWidgetState { ref style, .. } = *rectangle;
//                         let color = style.get_color(theme);
//                         match *style {
//                             ShapeStyle::Fill(_) => {
//                                 let kind = PrimitiveKind::Rectangle { color: color };
//                                 draw_primitive(new_primitive(kind, scizzor, rect));
//                             },
//                             ShapeStyle::Outline(ref line_style) => {
//                                 let (l, r, b, t) = rect.l_r_b_t();
//                                 points[0] = [l, b];
//                                 points[1] = [l, t];
//                                 points[2] = [r, t];
//                                 points[3] = [r, b];
//                                 points[4] = [l, b];
//                                 let cap = line_style.get_cap(theme);
//                                 let thickness = line_style.get_thickness(theme);
//                                 let kind = PrimitiveKind::Lines {
//                                     color: color,
//                                     cap: cap,
//                                     thickness: thickness,
//                                     points: &points[..5],
//                                 };
//                                 draw_primitive(new_primitive(kind, scizzor, rect));
//                             },
//                         }
//                     }
//                 },
// 
//                 primitive::shape::framed_rectangle::KIND => {
//                     if let Some(rectangle) = container.unique_widget_state::<::FramedRectangle>() {
//                         let graph::UniqueWidgetState { ref style, .. } = *rectangle;
//                         let frame = style.frame(theme);
//                         if frame > 0.0 {
//                             let frame_color = style.frame_color(theme);
//                             let kind = PrimitiveKind::Rectangle { color: frame_color };
//                             draw_primitive(new_primitive(kind, scizzor, rect));
//                         }
//                         let color = style.color(theme);
//                         let rect = rect.pad(frame);
//                         let kind = PrimitiveKind::Rectangle { color: color };
//                         draw_primitive(new_primitive(kind, scizzor, rect));
//                     }
//                 },
// 
//                 primitive::shape::oval::KIND => {
//                     if let Some(oval) = container.unique_widget_state::<::Oval>() {
//                         use std::f64::consts::PI;
//                         let graph::UniqueWidgetState { ref style, .. } = *oval;
// 
//                         let (x, y, w, h) = rect.x_y_w_h();
//                         let t = 2.0 * PI / CIRCLE_RESOLUTION as Scalar;
//                         let hw = w / 2.0;
//                         let hh = h / 2.0;
//                         let f = |i: Scalar| [x + hw * (t*i).cos(), y + hh * (t*i).sin()];
//                         for i in 0..NUM_POINTS {
//                             points[i] = f(i as f64);
//                         }
// 
//                         let color = style.get_color(theme);
//                         let points = &mut points[..NUM_POINTS];
//                         match *style {
//                             ShapeStyle::Fill(_) => {
//                                 let kind = PrimitiveKind::Polygon { color: color, points: points };
//                                 draw_primitive(new_primitive(kind, scizzor, rect));
//                             },
//                             ShapeStyle::Outline(ref line_style) => {
//                                 let cap = line_style.get_cap(theme);
//                                 let thickness = line_style.get_thickness(theme);
//                                 let kind = PrimitiveKind::Lines {
//                                     color: color,
//                                     cap: cap,
//                                     thickness: thickness,
//                                     points: points,
//                                 };
//                                 draw_primitive(new_primitive(kind, scizzor, rect));
//                             },
//                         }
//                     }
//                 },
// 
//                 primitive::shape::polygon::KIND => {
//                     use widget::primitive::shape::Style;
//                     use widget::primitive::shape::polygon::State;
//                     if let Some(polygon) = container.state_and_style::<State, Style>() {
//                         let graph::UniqueWidgetState { ref state, ref style } = *polygon;
// 
//                         let color = style.get_color(theme);
//                         let points = &state.points[..];
//                         match *style {
//                             ShapeStyle::Fill(_) => {
//                                 let kind = PrimitiveKind::Polygon { color: color, points: points };
//                                 draw_primitive(new_primitive(kind, scizzor, rect));
//                             },
//                             ShapeStyle::Outline(ref line_style) => {
//                                 let cap = line_style.get_cap(theme);
//                                 let thickness = line_style.get_thickness(theme);
//                                 let kind = PrimitiveKind::Lines {
//                                     color: color,
//                                     cap: cap,
//                                     thickness: thickness,
//                                     points: points,
//                                 };
//                                 draw_primitive(new_primitive(kind, scizzor, rect));
//                             },
//                         }
//                     }
//                 },
// 
//                 primitive::line::KIND => {
//                     if let Some(line) = container.unique_widget_state::<::Line>() {
//                         let graph::UniqueWidgetState { ref state, ref style } = *line;
//                         let color = style.get_color(theme);
//                         let cap = style.get_cap(theme);
//                         let thickness = style.get_thickness(theme);
//                         points[0] = state.start;
//                         points[1] = state.end;
//                         let points = &points[..2];
//                         let kind = PrimitiveKind::Lines {
//                             color: color,
//                             cap: cap,
//                             thickness: thickness,
//                             points: points,
//                         };
//                         draw_primitive(new_primitive(kind, scizzor, rect));
//                     }
//                 },
// 
//                 primitive::point_path::KIND => {
//                     use widget::primitive::point_path::{State, Style};
//                     if let Some(point_path) = container.state_and_style::<State, Style>() {
//                         let graph::UniqueWidgetState { ref state, ref style } = *point_path;
//                         let color = style.get_color(theme);
//                         let cap = style.get_cap(theme);
//                         let thickness = style.get_thickness(theme);
//                         let points = &state.points[..];
//                         let kind = PrimitiveKind::Lines {
//                             color: color,
//                             cap: cap,
//                             thickness: thickness,
//                             points: points,
//                         };
//                         draw_primitive(new_primitive(kind, scizzor, rect));
//                     }
//                 },
// 
//                 primitive::text::KIND => {
//                     if let Some(text) = container.unique_widget_state::<::Text>() {
//                         let graph::UniqueWidgetState { ref state, ref style } = *text;
//                         let font_id = match style.font_id(theme).or_else(|| fonts.ids().next()) {
//                             Some(id) => id,
//                             None => continue,
//                         };
//                         let font = match fonts.get(font_id) {
//                             Some(font) => font,
//                             None => continue,
//                         };
// 
//                         // Retrieve styling.
//                         let color = style.color(theme);
//                         let font_size = style.font_size(theme);
//                         let line_spacing = style.line_spacing(theme);
//                         let x_align = style.text_align(theme);
//                         let y_align = Align::End;
//                         let scale = text::pt_to_scale(font_size);
// 
//                         // Produce the text layout iterators.
//                         let line_infos = state.line_infos.iter().cloned();
//                         let lines = line_infos.clone().map(|info| &state.string[info.byte_range()]);
//                         let line_rects = text::line::rects(line_infos, font_size, rect,
//                                                            x_align, y_align, line_spacing);
// 
//                         // Clear the existing glyphs and fill the buffer with glyphs for this Text.
//                         positioned_glyphs.clear();
//                         for (line, line_rect) in lines.zip(line_rects) {
//                             let (x, y) = (line_rect.left() as f32, line_rect.top() as f32);
//                             let point = text::RtPoint { x: x, y: y };
//                             positioned_glyphs.extend(font.layout(line, scale, point).map(|g| g.standalone()));
//                         }
// 
//                         // Queue the glyphs to be cached.
//                         for glyph in positioned_glyphs.iter() {
//                             glyph_cache.queue_glyph(font_id.index(), glyph.clone());
//                         }
// 
//                         let kind = PrimitiveKind::Text {
//                             color: color,
//                             glyph_cache: glyph_cache,
//                             positioned_glyphs: positioned_glyphs,
//                             font_id: font_id,
//                         };
//                         draw_primitive(new_primitive(kind, scizzor, rect));
//                     }
//                 },
// 
//                 primitive::image::KIND => {
//                     use widget::primitive::image::{State, Style};
//                     if let Some(image) = container.state_and_style::<State, Style>() {
//                         let graph::UniqueWidgetState { ref state, ref style } = *image;
//                         let maybe_color = style.maybe_color(theme);
//                         let kind = PrimitiveKind::Image {
//                             maybe_color: maybe_color,
//                             texture_id: state.texture_id,
//                             source_rect: state.src_rect,
//                         };
//                         draw_primitive(new_primitive(kind, scizzor, rect));
//                     }
//                 },
// 
//                 // Return an `Other` variant for all non-primitive widgets.
//                 _ => {
//                     let kind = PrimitiveKind::Other(container);
//                     draw_primitive(new_primitive(kind, scizzor, rect));
//                 },
//             }
//         }
//     }
//         
// }



// impl<'a> Vertices<'a> {
// 
//     /// Construct a new `Vertices` iterator.
//     ///
//     /// Allocate and zero at least the first six elements so that we don't have to check the size
//     /// for triangles or rectangles.
//     pub fn new(primitives: Primitives<'a>) -> Self {
//         Vertices {
//             primitives: primitives,
//             vertices: vec![[0.0, 0.0]; 6],
//         }
//     }
// 
//     /// Yield the slice of vertices for the next primitive.
//     pub fn next(&mut self) -> &[[Scalar; 2]] {
//         use piston_graphics::triangulation;
// 
//         let Vertices { ref mut primitives, ref mut vertices } = *self;
// 
//         const IDENTITY: [[f32; 3]; 2] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
// 
//         fn tri_rectangle(Rect { x, y }: Rect, color: Color, vertices: &mut [[Scalar; 2]]) {
//             vertices[0] = [x.start, y.start];
//             vertices[1] = [x.end, y.start];
//             vertices[2] = [x.start, y.end];
//             vertices[3] = [x.end, y.start];
//             vertices[4] = [x.end, y.end];
//             vertices[5] = [x.start, y.end];
//         }
// 
//         primitives.next().map(|prim| {
// 
//             match prim.kind {
// 
//                 PrimitiveKind::Rectangle(state, style) => {
//                     match *style {
//                         ShapeStyle::Fill(_) => {
//                             let color = style.get_color(theme);
//                         },
//                         ShapeStyle::Outline(ref line_style) => {
//                         },
//                     }
// 
//                 },
// 
//                 PrimitiveKind::FramedRectangle(state, style) => {
//                 },
// 
//                 PrimitiveKind::Oval(state, style) => {
//                 },
// 
//                 PrimitiveKind::Polygon(state, style) => {
//                 },
// 
//                 PrimitiveKind::Line(state, style) => {
//                 },
// 
//                 PrimitiveKind::PointPath(state, style) => {
//                 },
// 
//                 PrimitiveKind::Text(state, style) => {
//                 },
// 
//                 PrimitiveKind::Image(state, style) => {
//                 },
//             }
//         })
//     }
// }
