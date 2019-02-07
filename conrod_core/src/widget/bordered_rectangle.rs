//! The `BorderedRectangle` widget and related items.

use {
    Color,
    Colorable,
    Dimensions,
    Borderable,
    Point,
    Positionable,
    Rect,
    Scalar,
    Sizeable,
    Widget,
};
use widget;
use widget::triangles::Triangle;


/// A filled rectangle widget that may or may not have some border.
#[derive(Copy, Clone, Debug, WidgetCommon_)]
pub struct BorderedRectangle {
    /// Data necessary and common for all widget builder types.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// Unique styling for the **BorderedRectangle**.
    pub style: Style,
}

widget_ids! {
    struct Ids {
        border,
        rectangle,
    }
}

/// Unique styling for the **BorderedRectangle** widget.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// Shape styling for the inner rectangle.
    #[conrod(default = "theme.shape_color")]
    pub color: Option<Color>,
    /// The thickness of the border.
    #[conrod(default = "theme.border_width")]
    pub border: Option<Scalar>,
    /// The color of the border.
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
}

/// Unique state for the `BorderedRectangle`.
pub struct State {
    ids: Ids,
}

impl BorderedRectangle {

    /// Build a new **BorderedRectangle**.
    pub fn new(dim: Dimensions) -> Self {
        BorderedRectangle {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
        }.wh(dim)
    }

    builder_method!(pub with_style { style = Style });

}


impl Widget for BorderedRectangle {
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the Rectangle.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, style, rect, ui, .. } = args;

        let border = style.border(&ui.theme);
        if let Some(triangles) = border_triangles(rect, border) {
            let border_color = style.border_color(&ui.theme);
            widget::Triangles::single_color(border_color, triangles.iter().cloned())
                .with_bounding_rect(rect)
                .parent(id)
                .graphics_for(id)
                .set(state.ids.border, ui);
        }

        let color = style.color(&ui.theme);
        widget::Rectangle::fill(rect.pad(border).dim())
            .xy(rect.xy())
            .color(color)
            .parent(id)
            .graphics_for(id)
            .set(state.ids.rectangle, ui);
    }

}


impl Colorable for BorderedRectangle {
    builder_method!(color { style.color = Some(Color) });
}


impl Borderable for BorderedRectangle {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}


/// The eight triangles that describe a rectangular border.
///
/// `rect` specifies the outer rectangle and `border` specifies the thickness of the border.
///
/// Returns `None` if `border` is less than or equal to `0`.
pub fn border_triangles(rect: Rect, border: Scalar) -> Option<[Triangle<Point>; 8]> {
    if border <= 0.0 {
        return None;
    }

    // Pad the edges so that the line does not exceed the bounding rect.
    let (l, r, b, t) = rect.l_r_b_t();
    let l_pad = l + border;
    let r_pad = r - border;
    let b_pad = b + border;
    let t_pad = t - border;

    // The four quads that make up the border.
    let r1 = [[l, t], [r_pad, t], [r_pad, t_pad], [l, t_pad]];
    let r2 = [[r_pad, t], [r, t], [r, b_pad], [r_pad, b_pad]];
    let r3 = [[l_pad, b_pad], [r, b_pad], [r, b], [l_pad, b]];
    let r4 = [[l, t_pad], [l_pad, t_pad], [l_pad, b], [l, b]];

    let (r1a, r1b) = widget::triangles::from_quad(r1);
    let (r2a, r2b) = widget::triangles::from_quad(r2);
    let (r3a, r3b) = widget::triangles::from_quad(r3);
    let (r4a, r4b) = widget::triangles::from_quad(r4);

    Some([r1a, r1b, r2a, r2b, r3a, r3b, r4a, r4b])
}

/// An iterator yielding triangles for a rounded border.
///
/// Clamps the thickness of the border to half the smallest dimension of the rectangle to
/// ensure the bounds of the `rect` are not exceeded.
pub fn rounded_border_triangles(
    rect: Rect,
    thickness: Scalar,
    radius: Scalar,
    corner_resolution: usize,
) -> RoundedBorderTriangles {
    RoundedBorderTriangles::new(rect, thickness, radius, corner_resolution)
}

/// An iterator yielding triangles for a rounded border.
#[derive(Clone)]
pub struct RoundedBorderTriangles {
    outer: widget::rounded_rectangle::Points,
    inner: widget::rounded_rectangle::Points,
    outer_end: Option<Point>,
    inner_end: Option<Point>,
    last_points: [Point; 2],
    is_next_outer: bool,
}

impl RoundedBorderTriangles {
    /// Constructor for an iterator yielding triangles for a rounded border.
    ///
    /// Clamps the thickness of the border to half the smallest dimension of the rectangle to
    /// ensure the bounds of the `rect` are not exceeded.
    pub fn new(
        rect: Rect,
        mut thickness: Scalar,
        radius: Scalar,
        corner_resolution: usize,
    ) -> Self {
        thickness = {
            let (w, h) = rect.w_h();
            thickness.min(w * 0.5).min(h * 0.5)
        };
        let inner_rect = rect.pad(thickness);
        let mut outer = widget::rounded_rectangle::points(rect, radius, corner_resolution);
        let mut inner = widget::rounded_rectangle::points(inner_rect, radius, corner_resolution);
        // A rounded_rectangle should always yield at least four points.
        let last_outer = outer.next().unwrap();
        let last_inner = inner.next().unwrap();
        let outer_end = Some(last_outer);
        let inner_end = Some(last_inner);
        let last_points = [last_outer, last_inner];
        let is_next_outer = true;
        RoundedBorderTriangles { outer, inner, is_next_outer, last_points, outer_end, inner_end }
    }
}

impl Iterator for RoundedBorderTriangles {
    type Item = Triangle<Point>;
    fn next(&mut self) -> Option<Self::Item> {
        let next_point = match self.is_next_outer {
            true => self.outer.next().or_else(|| self.outer_end.take()),
            false => self.inner.next().or_else(|| self.inner_end.take()),
        };
        next_point.map(|c| {
            self.is_next_outer = !self.is_next_outer;
            let a = self.last_points[0];
            let b = self.last_points[1];
            self.last_points[0] = b;
            self.last_points[1] = c;
            Triangle([a, b, c])
        })
    }
}
