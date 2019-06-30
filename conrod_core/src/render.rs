//! Conrod's generic graphics backend.
//!
//! **Note:** Conrod currently uses Piston's generic [graphics
//! crate](https://github.com/PistonDevelopers/graphics) (and specifically the
//! [**Graphics**](http://docs.piston.rs/graphics/graphics/trait.Graphics.html))
//! trait to enable genericity over custom user backends. This dependency may change in the near
//! future in favour of a simplified conrod-specific graphics and character caching backend trait.
//!
//! This is the only module in which the src graphics crate will be used directly.


use {Color, FontSize, Point, Rect, Scalar};
use color;
use graph::{self, Graph};
use image;
use position::{Align, Dimensions};
use std;
use text;
use theme::Theme;
use widget::{self, Widget};
use widget::triangles::{ColoredPoint, Triangle};


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
    /// A buffer to use for triangulating polygons and lines for the `Triangles`.
    triangles: Vec<Triangle<Point>>,
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
    triangles_single_color: Vec<Triangle<Point>>,
    triangles_multi_color: Vec<Triangle<ColoredPoint>>,
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

    /// A series of consecutive `Triangles` that are all the same color.
    TrianglesSingleColor {
        /// The color of all triangles.
        color: color::Rgba,
        /// An ordered slice of triangles.
        triangles: &'a [Triangle<Point>]
    },

    /// A series of consecutive `Triangles` with unique colors per vertex.
    ///
    /// This variant is produced by the general purpose `Triangles` primitive widget.
    TrianglesMultiColor {
        /// An ordered slice of multicolored triangles.
        triangles: &'a [Triangle<ColoredPoint>]
    },

    /// A single `Image`, produced by the primitive `Image` widget.
    Image {
        /// The unique identifier of the image that will be drawn.
        image_id: image::Id,
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
    justify: text::Justify,
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
    TrianglesSingleColor {
        color: color::Rgba,
        triangle_range: std::ops::Range<usize>,
    },
    TrianglesMultiColor {
        triangle_range: std::ops::Range<usize>,
    },
    Image {
        image_id: image::Id,
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
    justify: text::Justify,
    y_align: Align,
    line_spacing: Scalar,
}

/// An iterator-like type for yielding `Primitive`s from an `OwnedPrimitives`.
pub struct WalkOwnedPrimitives<'a> {
    primitives: std::slice::Iter<'a, OwnedPrimitive>,
    triangles_single_color: &'a [Triangle<Point>],
    triangles_multi_color: &'a [Triangle<ColoredPoint>],
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
            justify,
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
                                           justify, y_align, line_spacing);

        // Clear the existing glyphs and fill the buffer with glyphs for this Text.
        positioned_glyphs.clear();
        let scale = text::f32_pt_to_scale(font_size as f32 * dpi_factor);
        for (line, line_rect) in lines.zip(line_rects) {
            let (x, y) = (trans_x(line_rect.left()) as f32, trans_y(line_rect.bottom()) as f32);
            let point = text::rt::Point { x: x, y: y };
            positioned_glyphs.extend(font.layout(line, scale, point).map(|g| g.standalone()));
        }

        positioned_glyphs
    }

}


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
            triangles: Vec::new(),
            positioned_glyphs: Vec::new(),
        }
    }

    /// Yield the next `Primitive` for rendering.
    pub fn next(&mut self) -> Option<Primitive> {
        let Primitives {
            ref mut crop_stack,
            ref mut depth_order,
            ref mut triangles,
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

            type TrianglesSingleColorState =
                widget::triangles::State<Vec<widget::triangles::Triangle<Point>>>;
            type TrianglesMultiColorState =
                widget::triangles::State<Vec<widget::triangles::Triangle<(Point, color::Rgba)>>>;

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
                            let array = [
                                [l, b],
                                [l, t],
                                [r, t],
                                [r, b],
                                [l, b],
                            ];
                            let cap = line_style.get_cap(theme);
                            let thickness = line_style.get_thickness(theme);
                            let points = array.iter().cloned();
                            let triangles = match widget::point_path::triangles(points, cap, thickness) {
                                None => &[],
                                Some(iter) => {
                                    triangles.extend(iter);
                                    &triangles[..]
                                },
                            };
                            let kind = PrimitiveKind::TrianglesSingleColor {
                                color: color.to_rgb(),
                                triangles: &triangles,
                            };
                            return Some(new_primitive(id, kind, scizzor, rect));
                        },
                    }
                }

            } else if container.type_id == std::any::TypeId::of::<TrianglesSingleColorState>() {
                type Style = widget::triangles::SingleColor;
                if let Some(tris) = container.state_and_style::<TrianglesSingleColorState, Style>() {
                    let graph::UniqueWidgetState { ref state, ref style } = *tris;
                    let widget::triangles::SingleColor(color) = *style;
                    let kind = PrimitiveKind::TrianglesSingleColor {
                        color: color,
                        triangles: &state.triangles,
                    };
                    return Some(new_primitive(id, kind, scizzor, rect));
                }

            } else if container.type_id == std::any::TypeId::of::<TrianglesMultiColorState>() {
                type Style = widget::triangles::MultiColor;
                if let Some(tris) = container.state_and_style::<TrianglesMultiColorState, Style>() {
                    let graph::UniqueWidgetState { ref state, .. } = *tris;
                    let kind = PrimitiveKind::TrianglesMultiColor { triangles: &state.triangles };
                    return Some(new_primitive(id, kind, scizzor, rect));
                }

            } else if container.type_id == state_type_id::<widget::Oval<widget::oval::Full>>() {
                if let Some(oval) = container.unique_widget_state::<widget::Oval<widget::oval::Full>>() {
                    let graph::UniqueWidgetState { ref style, ref state } = *oval;
                    triangles.clear();
                    let points = widget::oval::circumference(rect, state.resolution);
                    let color = style.get_color(theme);
                    match *style {

                        ShapeStyle::Fill(_) => {
                            let triangles = {
                                triangles.extend(points.triangles());
                                &triangles[..]
                            };
                            let kind = PrimitiveKind::TrianglesSingleColor {
                                color: color.to_rgb(),
                                triangles: &triangles,
                            };
                            return Some(new_primitive(id, kind, scizzor, rect));
                        },

                        ShapeStyle::Outline(ref line_style) => {
                            let cap = line_style.get_cap(theme);
                            let thickness = line_style.get_thickness(theme);
                            let triangles = match widget::point_path::triangles(points, cap, thickness) {
                                None => &[],
                                Some(iter) => {
                                    triangles.extend(iter);
                                    &triangles[..]
                                },
                            };
                            let kind = PrimitiveKind::TrianglesSingleColor {
                                color: color.to_rgb(),
                                triangles: &triangles,
                            };
                            return Some(new_primitive(id, kind, scizzor, rect));
                        },
                    }
                }

            // Oval subsection.
            } else if container.type_id == state_type_id::<widget::Oval<widget::oval::Section>>() {
                if let Some(oval) = container.unique_widget_state::<widget::Oval<widget::oval::Section>>() {
                    let graph::UniqueWidgetState { ref style, ref state } = *oval;
                    triangles.clear();
                    let points = widget::oval::circumference(rect, state.resolution)
                        .section(state.section.radians)
                        .offset_radians(state.section.offset_radians);
                    let color = style.get_color(theme);
                    match *style {

                        ShapeStyle::Fill(_) => {
                            let triangles = {
                                triangles.extend(points.triangles());
                                &triangles[..]
                            };
                            let kind = PrimitiveKind::TrianglesSingleColor {
                                color: color.to_rgb(),
                                triangles: &triangles,
                            };
                            return Some(new_primitive(id, kind, scizzor, rect));
                        },

                        ShapeStyle::Outline(ref line_style) => {
                            use std::iter::once;
                            let cap = line_style.get_cap(theme);
                            let thickness = line_style.get_thickness(theme);
                            let middle = rect.xy();
                            let points = once(middle).chain(points).chain(once(middle));
                            let triangles = match widget::point_path::triangles(points, cap, thickness) {
                                None => &[],
                                Some(iter) => {
                                    triangles.extend(iter);
                                    &triangles[..]
                                },
                            };
                            let kind = PrimitiveKind::TrianglesSingleColor {
                                color: color.to_rgb(),
                                triangles: &triangles,
                            };
                            return Some(new_primitive(id, kind, scizzor, rect));
                        },
                    }
                }

            } else if container.type_id == std::any::TypeId::of::<PolygonState>() {
                use widget::primitive::shape::Style;
                if let Some(polygon) = container.state_and_style::<PolygonState, Style>() {
                    let graph::UniqueWidgetState { ref state, ref style } = *polygon;
                    triangles.clear();

                    let color = style.get_color(theme);
                    let points = state.points.iter().cloned();
                    
                    match *style {

                        ShapeStyle::Fill(_) => {
                            let triangles = match widget::polygon::triangles(points) {
                                None => &[],
                                Some(iter) => {
                                    triangles.extend(iter);
                                    &triangles[..]
                                },
                            };
                            
                            let kind = PrimitiveKind::TrianglesSingleColor {
                                color: color.to_rgb(),
                                triangles: &triangles,
                            };
                            return Some(new_primitive(id, kind, scizzor, rect));
                        },

                        ShapeStyle::Outline(ref line_style) => {
                            let cap = line_style.get_cap(theme);
                            let thickness = line_style.get_thickness(theme);
                            let triangles = match widget::point_path::triangles(points, cap, thickness) {
                                None => &[],
                                Some(iter) => {
                                    triangles.extend(iter);
                                    &triangles[..]
                                },
                            };
                            let kind = PrimitiveKind::TrianglesSingleColor {
                                color: color.to_rgb(),
                                triangles: &triangles,
                            };
                            return Some(new_primitive(id, kind, scizzor, rect));
                        },
                    }
                }

            } else if container.type_id == state_type_id::<widget::Line>() {
                if let Some(line) = container.unique_widget_state::<widget::Line>() {
                    let graph::UniqueWidgetState { ref state, ref style } = *line;
                    triangles.clear();
                    let color = style.get_color(theme);
                    let cap = style.get_cap(theme);
                    let thickness = style.get_thickness(theme);
                    let points = std::iter::once(state.start).chain(std::iter::once(state.end));
                    let triangles = match widget::point_path::triangles(points, cap, thickness) {
                        None => &[],
                        Some(iter) => {
                            triangles.extend(iter);
                            &triangles[..]
                        },
                    };
                    let kind = PrimitiveKind::TrianglesSingleColor {
                        color: color.to_rgb(),
                        triangles: triangles,
                    };
                    return Some(new_primitive(id, kind, scizzor, rect));
                }

            } else if container.type_id == std::any::TypeId::of::<PointPathState>() {
                if let Some(point_path) = container.state_and_style::<PointPathState, PointPathStyle>() {
                    let graph::UniqueWidgetState { ref state, ref style } = *point_path;
                    triangles.clear();
                    let color = style.get_color(theme);
                    let cap = style.get_cap(theme);
                    let thickness = style.get_thickness(theme);
                    let points = state.points.iter().map(|&t| t);
                    let triangles = match widget::point_path::triangles(points, cap, thickness) {
                        None => &[],
                        Some(iter) => {
                            triangles.extend(iter);
                            &triangles[..]
                        },
                    };
                    let kind = PrimitiveKind::TrianglesSingleColor {
                        color: color.to_rgb(),
                        triangles: triangles,
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
                    let justify = style.justify(theme);
                    let y_align = Align::End;

                    let text = Text {
                        positioned_glyphs: positioned_glyphs,
                        window_dim: window_rect.dim(),
                        text: &state.string,
                        line_infos: &state.line_infos,
                        font: font,
                        font_size: font_size,
                        rect: rect,
                        justify: justify,
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
                        image_id: state.image_id,
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
        let mut primitive_triangles_multi_color = Vec::new();
        let mut primitive_triangles_single_color = Vec::new();
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

                PrimitiveKind::TrianglesSingleColor { color, triangles } => {
                    let start = primitive_triangles_single_color.len();
                    primitive_triangles_single_color.extend(triangles.iter().cloned());
                    let end = primitive_triangles_single_color.len();
                    let kind = OwnedPrimitiveKind::TrianglesSingleColor {
                        color: color,
                        triangle_range: start..end,
                    };
                    primitives.push(new(kind));
                },

                PrimitiveKind::TrianglesMultiColor { triangles } => {
                    let start = primitive_triangles_multi_color.len();
                    primitive_triangles_multi_color.extend(triangles.iter().cloned());
                    let end = primitive_triangles_multi_color.len();
                    let kind = OwnedPrimitiveKind::TrianglesMultiColor {
                        triangle_range: start..end,
                    };
                    primitives.push(new(kind));
                },

                PrimitiveKind::Image { image_id, color, source_rect } => {
                    let kind = OwnedPrimitiveKind::Image {
                        image_id: image_id,
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
                        justify,
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
                        justify: justify,
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
            triangles_single_color: primitive_triangles_single_color,
            triangles_multi_color: primitive_triangles_multi_color,
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
            ref triangles_single_color,
            ref triangles_multi_color,
            ref line_infos,
            ref texts_string,
            max_glyphs,
        } = *self;
        WalkOwnedPrimitives {
            primitives: primitives.iter(),
            triangles_single_color: triangles_single_color,
            triangles_multi_color: triangles_multi_color,
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
            triangles_single_color,
            triangles_multi_color,
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

                OwnedPrimitiveKind::TrianglesSingleColor { color, ref triangle_range } => {
                    let kind = PrimitiveKind::TrianglesSingleColor {
                        color: color,
                        triangles: &triangles_single_color[triangle_range.clone()],
                    };
                    new(kind)
                },

                OwnedPrimitiveKind::TrianglesMultiColor { ref triangle_range } => {
                    let kind = PrimitiveKind::TrianglesMultiColor {
                        triangles: &triangles_multi_color[triangle_range.clone()],
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
                        justify,
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
                        justify: justify,
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

                OwnedPrimitiveKind::Image { image_id, color, source_rect } => {
                    let kind = PrimitiveKind::Image {
                        image_id: image_id,
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
