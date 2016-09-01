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
use graph::{self, Graph};
use std;
use text;
use theme::Theme;
use widget::{self, primitive, Widget};


/// An iterator-like type that yields a reference to each primitive in order of depth for
/// rendering.
///
/// This type is produced by the `Ui::draw` and `Ui::draw_if_changed` methods.
///
/// This type borrows data from the `Ui` in order to lazily produce each `Primitive`. If you
/// require ownership over the sequence of primitives, consider using the `OwnedPrimitives` type.
/// The `OwnedPrimitives` type can be produced by calling the `Primitives::owned` method.
pub struct Primitives<'a> {
    crop_stack: Vec<(widget::Id, Rect)>,
    depth_order: std::slice::Iter<'a, widget::Id>,
    graph: &'a Graph,
    theme: &'a Theme,
    fonts: &'a text::font::Map,
    window_rect: Rect,
    /// The point slice to use for the `Lines` and `Polygon` primitives.
    points: Vec<Point>,
    /// The slice of rusttype `PositionedGlyph`s to re-use for the `Text` primitive.
    positioned_glyphs: Vec<text::PositionedGlyph>,
}

/// An owned alternative to the `Primitives` type.
///
/// This is particularly useful for sending rendering data across threads.
///
/// Produce an `OwnedPrimitives` instance via the `Primitives::owned` method.
#[derive(Clone)]
pub struct OwnedPrimitives {
    primitives: Vec<OwnedPrimitive>,
    points: Vec<Point>,
    max_glyphs: usize,
    line_infos: Vec<text::line::Info>,
    texts_string: String,
}


/// A trait that allows the user to remain generic over types yielding `Primitive`s.
///
/// This trait is implemented for both the `Primitives` and `WalkOwnedPrimitives` types.
pub trait PrimitiveWalker {
    /// Yield the next `Primitive` in order of depth, bottom to top.
    fn next_primitive(&mut self) -> Option<Primitive>;
}

impl<'a> PrimitiveWalker for Primitives<'a> {
    fn next_primitive(&mut self) -> Option<Primitive> {
        self.next()
    }
}

impl<'a> PrimitiveWalker for WalkOwnedPrimitives<'a> {
    fn next_primitive(&mut self) -> Option<Primitive> {
        self.next()
    }
}


/// Data required for rendering a single primitive widget.
pub struct Primitive<'a> {
    /// The id of the widget within the widget graph.
    pub id: widget::Id,
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
    /// These are produced by the `Rectangle` and `BorderedRectangle` primitive widgets. A `Filled`
    /// `Rectangle` widget produces a single `Rectangle`. The `BorderedRectangle` produces two
    /// `Rectangle`s, the first for the outer border and the second for the inner on top.
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
        /// When `Some`, colours the `Image`. When `None`, the `Image` uses its regular colours.
        color: Option<Color>,
        /// The area of the texture that will be drawn to the `Image`'s `Rect`.
        source_rect: Option<Rect>,
    },

    /// A single block of `Text`, produced by the primitive `Text` widget.
    Text {
        /// The colour of the `Text`.
        color: Color,
        /// All glyphs within the `Text` laid out in their correct positions in order from top-left
        /// to bottom right.
        text: Text<'a>,
        /// The unique identifier for the font, useful for the `glyph_cache.rect_for(id, glyph)`
        /// method when using the `conrod::text::GlyphCache` (rusttype's GPU `Cache`).
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

/// A type used for producing a `PositionedGlyph` iterator.
///
/// We produce this type rather than the `&[PositionedGlyph]`s directly so that we can properly
/// handle "HiDPI" scales when caching glyphs.
pub struct Text<'a> {
    positioned_glyphs: &'a mut Vec<text::PositionedGlyph>,
    window_dim: Dimensions,
    text: &'a str,
    line_infos: &'a [text::line::Info],
    font: &'a text::Font,
    font_size: FontSize,
    rect: Rect,
    x_align: Align,
    y_align: Align,
    line_spacing: Scalar,
}


#[derive(Clone)]
struct OwnedPrimitive {
    id: widget::Id,
    kind: OwnedPrimitiveKind,
    scizzor: Rect,
    rect: Rect,
}

#[derive(Clone)]
enum OwnedPrimitiveKind {
    Rectangle {
        color: Color,
    },
    Polygon {
        color: Color,
        point_range: std::ops::Range<usize>,
    },
    Lines {
        color: Color,
        cap: primitive::line::Cap,
        thickness: Scalar,
        point_range: std::ops::Range<usize>,
    },
    Image {
        color: Option<Color>,
        source_rect: Option<Rect>,
    },
    Text {
        color: Color,
        font_id: text::font::Id,
        text: OwnedText,
    },
}

#[derive(Clone)]
struct OwnedText {
    str_byte_range: std::ops::Range<usize>,
    line_infos_range: std::ops::Range<usize>,
    window_dim: Dimensions,
    font: text::Font,
    font_size: FontSize,
    rect: Rect,
    x_align: Align,
    y_align: Align,
    line_spacing: Scalar,
}

/// An iterator-like type for yielding `Primitive`s from an `OwnedPrimitives`.
pub struct WalkOwnedPrimitives<'a> {
    primitives: std::slice::Iter<'a, OwnedPrimitive>,
    points: &'a [Point],
    line_infos: &'a [text::line::Info],
    texts_str: &'a str,
    positioned_glyphs: Vec<text::PositionedGlyph>,
}


impl<'a> Text<'a> {

    /// Produces a list of `PositionedGlyph`s which may be used to cache and render the text.
    ///
    /// `dpi_factor`, aka "dots per inch factor" is a multiplier representing the density of
    /// the display's pixels. The `Scale` of the font will be multiplied by this factor in order to
    /// ensure that each `PositionedGlyph`'s `pixel_bounding_box` is accurate and that the GPU
    /// cache receives glyphs of a size that will display correctly on displays regardless of DPI.
    ///
    /// Note that conrod does not require this factor when instantiating `Text` widgets and laying
    /// out text. This is because conrod positioning uses a "pixel-agnostic" `Scalar` value
    /// representing *perceived* distances for its positioning and layout, rather than pixel
    /// values. During rendering however, the pixel density must be known
    pub fn positioned_glyphs(self, dpi_factor: f32) -> &'a [text::PositionedGlyph] {
        let Text {
            positioned_glyphs,
            window_dim,
            text,
            line_infos,
            font,
            font_size,
            rect,
            x_align,
            y_align,
            line_spacing,
        } = self;

        // Convert conrod coordinates to pixel coordinates.
        let trans_x = |x: Scalar| (x + window_dim[0] / 2.0) * dpi_factor as Scalar;
        let trans_y = |y: Scalar| ((-y) + window_dim[1] / 2.0) * dpi_factor as Scalar;

        // Produce the text layout iterators.
        let line_infos = line_infos.iter().cloned();
        let lines = line_infos.clone().map(|info| &text[info.byte_range()]);
        let line_rects = text::line::rects(line_infos, font_size, rect,
                                           x_align, y_align, line_spacing);

        // Clear the existing glyphs and fill the buffer with glyphs for this Text.
        positioned_glyphs.clear();
        let scale = text::pt_to_scale((font_size as f32 * dpi_factor) as FontSize);
        for (line, line_rect) in lines.zip(line_rects) {
            let (x, y) = (trans_x(line_rect.left()) as f32, trans_y(line_rect.bottom()) as f32);
            let point = text::rt::Point { x: x, y: y };
            positioned_glyphs.extend(font.layout(line, scale, point).map(|g| g.standalone()));
        }

        positioned_glyphs
    }

}

const CIRCLE_RESOLUTION: usize = 50;
const NUM_POINTS: usize = CIRCLE_RESOLUTION + 1;


impl<'a> Primitives<'a> {

    /// Constructor for the `Primitives` iterator.
    pub fn new(graph: &'a Graph,
               depth_order: &'a [widget::Id],
               theme: &'a Theme,
               fonts: &'a text::font::Map,
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
        }
    }

    /// Yield the next `Primitive` for rendering.
    pub fn next(&mut self) -> Option<Primitive> {
        let Primitives {
            ref mut crop_stack,
            ref mut depth_order,
            ref mut points,
            ref mut positioned_glyphs,
            graph,
            theme,
            fonts,
            window_rect,
        } = *self;

        while let Some(widget) = next_widget(depth_order, graph, crop_stack, window_rect) {
            use widget::primitive::point_path::{State as PointPathState, Style as PointPathStyle};
            use widget::primitive::shape::polygon::{State as PolygonState};
            use widget::primitive::shape::Style as ShapeStyle;

            let (id, scizzor, container) = widget;
            let rect = container.rect;

            fn state_type_id<W>() -> std::any::TypeId
                where W: Widget,
            {
                std::any::TypeId::of::<W::State>()
            }

            // Extract the unique state and style from the container.
            if container.type_id == state_type_id::<widget::Rectangle>() {
                if let Some(rectangle) = container.unique_widget_state::<widget::Rectangle>() {
                    let graph::UniqueWidgetState { ref style, .. } = *rectangle;
                    let color = style.get_color(theme);
                    match *style {
                        ShapeStyle::Fill(_) => {
                            let kind = PrimitiveKind::Rectangle { color: color };
                            return Some(new_primitive(id, kind, scizzor, rect));
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
                            return Some(new_primitive(id, kind, scizzor, rect));
                        },
                    }
                }

            } else if container.type_id == state_type_id::<widget::Oval>() {
                if let Some(oval) = container.unique_widget_state::<widget::Oval>() {
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
                            return Some(new_primitive(id, kind, scizzor, rect));
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
                            return Some(new_primitive(id, kind, scizzor, rect));
                        },
                    }
                }

            } else if container.type_id == std::any::TypeId::of::<PolygonState>() {
                use widget::primitive::shape::Style;
                if let Some(polygon) = container.state_and_style::<PolygonState, Style>() {
                    let graph::UniqueWidgetState { ref state, ref style } = *polygon;

                    let color = style.get_color(theme);
                    let points = &state.points[..];
                    match *style {
                        ShapeStyle::Fill(_) => {
                            let kind = PrimitiveKind::Polygon { color: color, points: points };
                            return Some(new_primitive(id, kind, scizzor, rect));
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
                            return Some(new_primitive(id, kind, scizzor, rect));
                        },
                    }
                }

            } else if container.type_id == state_type_id::<widget::Line>() {
                if let Some(line) = container.unique_widget_state::<widget::Line>() {
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
                    return Some(new_primitive(id, kind, scizzor, rect));
                }

            } else if container.type_id == std::any::TypeId::of::<PointPathState>() {
                if let Some(point_path) = container.state_and_style::<PointPathState, PointPathStyle>() {
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
                    return Some(new_primitive(id, kind, scizzor, rect));
                }

            } else if container.type_id == state_type_id::<widget::Text>() {
                if let Some(text) = container.unique_widget_state::<widget::Text>() {
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

                    let text = Text {
                        positioned_glyphs: positioned_glyphs,
                        window_dim: window_rect.dim(),
                        text: &state.string,
                        line_infos: &state.line_infos,
                        font: font,
                        font_size: font_size,
                        rect: rect,
                        x_align: x_align,
                        y_align: y_align,
                        line_spacing: line_spacing,
                    };

                    let kind = PrimitiveKind::Text {
                        color: color,
                        text: text,
                        font_id: font_id,
                    };
                    return Some(new_primitive(id, kind, scizzor, rect));
                }

            } else if container.type_id == state_type_id::<widget::Image>() {
                use widget::primitive::image::{State, Style};
                if let Some(image) = container.state_and_style::<State, Style>() {
                    let graph::UniqueWidgetState { ref state, ref style } = *image;
                    let color = style.maybe_color(theme);
                    let kind = PrimitiveKind::Image {
                        color: color,
                        source_rect: state.src_rect,
                    };
                    return Some(new_primitive(id, kind, scizzor, rect));
                }

            // Return an `Other` variant for all non-primitive widgets.
            } else {
                let kind = PrimitiveKind::Other(container);
                return Some(new_primitive(id, kind, scizzor, rect));
            }
        }

        None
    }

    /// Collect the `Primitives` list into an owned collection.
    ///
    /// This is useful for sending `Ui` rendering data across threads in an efficient manner.
    pub fn owned(mut self) -> OwnedPrimitives {
        let mut primitives = Vec::with_capacity(self.depth_order.len());
        let mut primitive_points = Vec::new();
        let mut primitive_line_infos = Vec::new();
        let mut texts_string = String::new();
        let mut max_glyphs = 0;

        while let Some(Primitive { id, rect, scizzor, kind }) = self.next() {
            let new = |kind| OwnedPrimitive {
                id: id,
                rect: rect,
                scizzor: scizzor,
                kind: kind,
            };

            match kind {

                PrimitiveKind::Rectangle { color } => {
                    let kind = OwnedPrimitiveKind::Rectangle { color: color };
                    primitives.push(new(kind));
                },

                PrimitiveKind::Polygon { color, points } => {
                    let start = primitive_points.len();
                    primitive_points.extend(points.iter().cloned());
                    let end = primitive_points.len();
                    let kind = OwnedPrimitiveKind::Polygon {
                        color: color,
                        point_range: start..end,
                    };
                    primitives.push(new(kind));
                },

                PrimitiveKind::Lines { color, cap, thickness, points } => {
                    let start = primitive_points.len();
                    primitive_points.extend(points.iter().cloned());
                    let end = primitive_points.len();
                    let kind = OwnedPrimitiveKind::Lines {
                        color: color,
                        cap: cap,
                        thickness: thickness,
                        point_range: start..end,
                    };
                    primitives.push(new(kind));
                },

                PrimitiveKind::Image { color, source_rect } => {
                    let kind = OwnedPrimitiveKind::Image {
                        color: color,
                        source_rect: source_rect,
                    };
                    primitives.push(new(kind));
                },

                PrimitiveKind::Text { color, font_id, text } => {
                    let Text {
                        window_dim,
                        text,
                        line_infos,
                        font,
                        font_size,
                        rect,
                        x_align,
                        y_align,
                        line_spacing,
                        ..
                    } = text;

                    // Keep a rough estimate of the maximum number of glyphs so that we know what
                    // capacity we should allocate the `PositionedGlyph` buffer with.
                    max_glyphs = std::cmp::max(max_glyphs, text.len());

                    // Pack the `texts_string`.
                    let start_str_byte = texts_string.len();
                    texts_string.push_str(text);
                    let end_str_byte = texts_string.len();

                    // Pack the `line_infos`.
                    let start_line_info_idx = primitive_line_infos.len();
                    primitive_line_infos.extend(line_infos.iter().cloned());
                    let end_line_info_idx = primitive_line_infos.len();

                    let owned_text = OwnedText {
                        str_byte_range: start_str_byte..end_str_byte,
                        line_infos_range: start_line_info_idx..end_line_info_idx,
                        window_dim: window_dim,
                        font: font.clone(),
                        font_size: font_size,
                        rect: rect,
                        x_align: x_align,
                        y_align: y_align,
                        line_spacing: line_spacing,
                    };

                    let kind = OwnedPrimitiveKind::Text {
                        color: color,
                        font_id: font_id,
                        text: owned_text,
                    };
                    primitives.push(new(kind));
                },

                // TODO: Not sure how we should handle this yet.
                PrimitiveKind::Other(_) => (),

            }
        }

        OwnedPrimitives {
            primitives: primitives,
            points: primitive_points,
            max_glyphs: max_glyphs,
            line_infos: primitive_line_infos,
            texts_string: texts_string,
        }
    }

}


impl OwnedPrimitives {

    /// Produce an iterator-like type for yielding `Primitive`s.
    pub fn walk(&self) -> WalkOwnedPrimitives {
        let OwnedPrimitives {
            ref primitives,
            ref points,
            ref line_infos,
            ref texts_string,
            max_glyphs,
        } = *self;
        WalkOwnedPrimitives {
            primitives: primitives.iter(),
            points: points,
            line_infos: line_infos,
            texts_str: texts_string,
            positioned_glyphs: Vec::with_capacity(max_glyphs),
        }
    }

}

impl<'a> WalkOwnedPrimitives<'a> {

    /// Yield the next `Primitive` in order or rendering depth, bottom to top.
    pub fn next(&mut self) -> Option<Primitive> {
        let WalkOwnedPrimitives {
            ref mut primitives,
            ref mut positioned_glyphs,
            points,
            line_infos,
            texts_str,
        } = *self;

        primitives.next().map(move |&OwnedPrimitive { id, rect, scizzor, ref kind }| {
            let new = |kind| Primitive {
                id: id,
                rect: rect,
                scizzor: scizzor,
                kind: kind,
            };

            match *kind {

                OwnedPrimitiveKind::Rectangle { color } => {
                    let kind = PrimitiveKind::Rectangle { color: color };
                    new(kind)
                },

                OwnedPrimitiveKind::Polygon { color, ref point_range } => {
                    let kind = PrimitiveKind::Polygon {
                        color: color,
                        points: &points[point_range.clone()],
                    };
                    new(kind)
                },

                OwnedPrimitiveKind::Lines { color, cap, thickness, ref point_range } => {
                    let kind = PrimitiveKind::Lines {
                        color: color,
                        cap: cap,
                        thickness: thickness,
                        points: &points[point_range.clone()],
                    };
                    new(kind)
                },

                OwnedPrimitiveKind::Text { color, font_id, ref text } => {
                    let OwnedText {
                        ref str_byte_range,
                        ref line_infos_range,
                        ref font,
                        window_dim,
                        font_size,
                        rect,
                        x_align,
                        y_align,
                        line_spacing,
                    } = *text;

                    let text_str = &texts_str[str_byte_range.clone()];
                    let line_infos = &line_infos[line_infos_range.clone()];

                    let text = Text {
                        positioned_glyphs: positioned_glyphs,
                        window_dim: window_dim,
                        text: text_str,
                        line_infos: line_infos,
                        font: font,
                        font_size: font_size,
                        rect: rect,
                        x_align: x_align,
                        y_align: y_align,
                        line_spacing: line_spacing,
                    };

                    let kind = PrimitiveKind::Text {
                        color: color,
                        font_id: font_id,
                        text: text,
                    };
                    new(kind)
                },

                OwnedPrimitiveKind::Image { color, source_rect } => {
                    let kind = PrimitiveKind::Image {
                        color: color,
                        source_rect: source_rect,
                    };
                    new(kind)
                },
            }
        })
    }

}



/// Simplify the constructor for a `Primitive`.
fn new_primitive(id: widget::Id, kind: PrimitiveKind, scizzor: Rect, rect: Rect) -> Primitive {
    Primitive {
        id: id,
        kind: kind,
        scizzor: scizzor,
        rect: rect,
    }
}

/// Retrieves the next visible widget from the `depth_order`, updating the `crop_stack` as
/// necessary.
fn next_widget<'a>(depth_order: &mut std::slice::Iter<widget::Id>,
                   graph: &'a Graph,
                   crop_stack: &mut Vec<(widget::Id, Rect)>,
                   window_rect: Rect) -> Option<(widget::Id, Rect, &'a graph::Container)>
{
    while let Some(&id) = depth_order.next() {
        let container = match graph.widget(id) {
            Some(container) => container,
            None => continue,
        };

        // If we're currently using a cropped context and the current `crop_parent_idx` is
        // *not* a depth-wise parent of the widget at the current `idx`, we should pop that
        // cropped context from the stack as we are done with it.
        while let Some(&(crop_parent_idx, _)) = crop_stack.last() {
            if graph.does_recursive_depth_edge_exist(crop_parent_idx, id) {
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
            crop_stack.push((id, scizzor_rect));
        }

        // We only want to return primitives that are actually visible.
        let is_visible = container.rect.overlap(window_rect).is_some()
            && graph::algo::cropped_area_of_widget(graph, id).is_some();
        if !is_visible {
            continue;
        }

        return Some((id, scizzor, container));
    }

    None
}
