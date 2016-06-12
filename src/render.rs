//! Conrod's generic graphics backend.
//!
//! **Note:** Conrod currently uses Piston's generic [graphics
//! crate](https://github.com/PistonDevelopers/graphics) (and specifically the
//! [**Graphics**](http://docs.piston.rs/graphics/graphics/trait.Graphics.html))
//! trait to enable genericity over custom user backends. This dependency may change in the near
//! future in favour of a simplified conrod-specific graphics and character caching backend trait.
//!
//! This is the only module in which the piston graphics crate will be used directly.


use {Align, Color, Dimensions, FontSize, Point, Rect, Scalar};
use graph::{self, Container, Graph, NodeIndex};
use rusttype;
use std::any::Any;
use std::iter::once;
use std;
use text;
use theme::Theme;
use widget::primitive;


/// An iterator yielding a reference to each primitive in order of depth for rendering.
pub struct Primitives<'a> {
    crop_stack: Vec<(NodeIndex, Rect)>,
    depth_order: std::slice::Iter<'a, NodeIndex>,
    graph: &'a Graph,
    theme: &'a Theme,
    window_rect: Rect,
    /// The point slice to use for the `Lines` and `Polygon` primitives.
    points: Vec<Point>,
}

/// Data required for rendering a single primitive widget.
#[derive(Clone)]
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
#[derive(Clone)]
pub enum PrimitiveKind<'a> {
    Rectangle {
        color: Color,
    },
    Polygon {
        color: Color,
        points: &'a [Point],
    },
    Lines {
        color: Color,
        cap: primitive::line::Cap,
        thickness: Scalar,
        points: &'a [Point],
    },
    Image {
        texture_index: usize,
        source_rect: Option<Rect>,
    },
    Text {
        color: Color,
        text: &'a str,
        line_infos: &'a [text::line::Info],
        font_size: FontSize,
        line_spacing: Scalar,
        x_align: Align,
        y_align: Align,
    },
}

/// An iterator yielding vertices for each `Primitive` widget.
pub struct Vertices<'a> {
    primitives: Primitives<'a>,
    vertices: Vec<Point>,
}


const CIRCLE_RESOLUTION: usize = 50;
const NUM_POINTS: usize = CIRCLE_RESOLUTION + 1;


impl<'a> Primitives<'a> {
    /// Constructor for the `Primitives` iterator.
    pub fn new(graph: &'a Graph,
               depth_order: &'a [NodeIndex],
               theme: &'a Theme,
               window_dim: Dimensions) -> Self
    {
        Primitives {
            crop_stack: Vec::new(),
            depth_order: depth_order.iter(),
            graph: graph,
            theme: theme,
            window_rect: Rect::from_xy_dim([0.0, 0.0], window_dim),
            // Initialise the `points` `Vec` with at least as many points as there are in an
            // outlined `Rectangle`. This saves us from having to check the length of the buffer
            // before writing points for an `Oval` or `Rectangle`.
            points: vec![[0.0, 0.0]; NUM_POINTS],
        }
    }
}


impl<'a> Primitives<'a> {
    pub fn draw<F>(&mut self, mut draw_primitive: F)
        where F: FnMut(Primitive),
    {
        let Primitives {
            ref mut crop_stack,
            ref mut depth_order,
            graph,
            theme,
            window_rect,
            ref mut points,
        } = *self;

        while let Some(&node_index) = depth_order.next() {
            use widget::primitive::shape::Style as ShapeStyle;

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

            let rect = container.rect;

            // Simplify the constructor for a `Primitive`.
            fn new_primitive(kind: PrimitiveKind, scizzor: Rect, rect: Rect) -> Primitive {
                Primitive {
                    kind: kind,
                    scizzor: scizzor,
                    rect: rect,
                }
            }

            // Extract the unique state and style from the container.
            match container.kind {

                primitive::shape::rectangle::KIND => {
                    if let Some(rectangle) = container.unique_widget_state::<::Rectangle>() {
                        let graph::UniqueWidgetState { ref style, .. } = *rectangle;
                        let color = style.get_color(theme);
                        match *style {
                            ShapeStyle::Fill(_) => {
                                let kind = PrimitiveKind::Rectangle { color: color };
                                draw_primitive(new_primitive(kind, scizzor, rect));
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
                                draw_primitive(new_primitive(kind, scizzor, rect));
                            },
                        }
                    }
                },

                primitive::shape::framed_rectangle::KIND => {
                    if let Some(rectangle) = container.unique_widget_state::<::FramedRectangle>() {
                        let graph::UniqueWidgetState { ref style, .. } = *rectangle;
                        let frame = style.frame(theme);
                        if frame > 0.0 {
                            let frame_color = style.frame_color(theme);
                            let kind = PrimitiveKind::Rectangle { color: frame_color };
                            draw_primitive(new_primitive(kind, scizzor, rect));
                        }
                        let color = style.color(theme);
                        let rect = rect.pad(frame);
                        let kind = PrimitiveKind::Rectangle { color: color };
                        draw_primitive(new_primitive(kind, scizzor, rect));
                    }
                },

                primitive::shape::oval::KIND => {
                    if let Some(oval) = container.unique_widget_state::<::Oval>() {
                        use std::f64::consts::PI;
                        let graph::UniqueWidgetState { ref state, ref style } = *oval;

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
                                draw_primitive(new_primitive(kind, scizzor, rect));
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
                                draw_primitive(new_primitive(kind, scizzor, rect));
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
                                draw_primitive(new_primitive(kind, scizzor, rect));
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
                                draw_primitive(new_primitive(kind, scizzor, rect));
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
                        draw_primitive(new_primitive(kind, scizzor, rect));
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
                        draw_primitive(new_primitive(kind, scizzor, rect));
                    }
                },

                primitive::text::KIND => {
                    if let Some(text) = container.unique_widget_state::<::Text>() {
                        let graph::UniqueWidgetState { ref state, ref style } = *text;
                        let kind = PrimitiveKind::Text {
                            color: style.color(theme),
                            text: &state.string[..],
                            line_infos: &state.line_infos[..],
                            font_size: style.font_size(theme),
                            line_spacing: style.line_spacing(theme),
                            x_align: style.text_align(theme),
                            y_align: Align::End,
                        };
                        draw_primitive(new_primitive(kind, scizzor, rect));
                    }
                },

                primitive::image::KIND => {
                    use widget::primitive::image::{State, Style};
                    if let Some(image) = container.state_and_style::<State, Style>() {
                        let graph::UniqueWidgetState { ref state, ref style } = *image;
                        let kind = PrimitiveKind::Image {
                            texture_index: state.texture_index,
                            source_rect: state.src_rect,
                        };
                        draw_primitive(new_primitive(kind, scizzor, rect));
                    }
                },

                _ => (),
            }
        }
    }
        
}



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
