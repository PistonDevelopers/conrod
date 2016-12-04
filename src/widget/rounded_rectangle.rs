//! Similar to the `Rectangle` widget however is drawn with rounded corners.
//!
//! The roundedness of the corners is specified with a `radius`. This indicates the radius of the
//! circle used to draw the corners.

use {Color, Colorable, Dimensions, Point, Positionable, Scalar, Sizeable, Widget};
use std;
use super::primitive::shape::Style;
use widget;


/// Draws a rectangle with corners rounded via the given radius.
#[derive(Copy, Clone, Debug)]
pub struct RoundedRectangle {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// Unique styling for the **RoundedRectangle**.
    pub style: Style,
    /// The radius of the circle used to round each corner of the rectangle.
    pub radius: Scalar,
    /// The number of points in each corner of the circle used to draw the rounded corners.
    pub corner_resolution: usize,
}

widget_ids! {
    struct Ids { polygon }
}

/// The default resolution of the circle used to draw the rounded corners.
pub const DEFAULT_CORNER_RESOLUTION: usize = 12;

/// Unique state for the `RoundedRectangle`.
pub struct State {
    ids: Ids,
}

impl RoundedRectangle {

    /// Build a rounded rectangle with the given dimensions and style.
    pub fn styled(dim: Dimensions, radius: Scalar, style: Style) -> Self {
        RoundedRectangle {
            common: widget::CommonBuilder::new(),
            style: style,
            radius: radius,
            corner_resolution: DEFAULT_CORNER_RESOLUTION,
        }.wh(dim)
    }

    /// Build a new filled rounded rectangle.
    pub fn fill(dim: Dimensions, radius: Scalar) -> Self {
        RoundedRectangle::styled(dim, radius, Style::fill())
    }

    /// Build a new filled rounded rectangle widget filled with the given color.
    pub fn fill_with(dim: Dimensions, radius: Scalar, color: Color) -> Self {
        RoundedRectangle::styled(dim, radius, Style::fill_with(color))
    }

    /// Build a new outlined rounded rectangle widget.
    pub fn outline(dim: Dimensions, radius: Scalar) -> Self {
        RoundedRectangle::styled(dim, radius, Style::outline())
    }

    /// Build an outlined rounded rectangle rather than a filled one.
    pub fn outline_styled(dim: Dimensions, radius: Scalar, line_style: widget::line::Style) -> Self {
        RoundedRectangle::styled(dim, radius, Style::outline_styled(line_style))
    }

    /// The number of points in each corner of the circle used to draw the rounded corners.
    pub fn corner_resolution(mut self, res: usize) -> Self {
        self.corner_resolution = res;
        self
    }

}

impl Widget for RoundedRectangle {
    type State = State;
    type Style = Style;
    type Event = ();

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

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
        let RoundedRectangle { radius, corner_resolution, .. } = self;

        let (l, r, b, t) = rect.l_r_b_t();
        // The rectangle edges with the radius subtracted.
        let (in_l, in_r, in_b, in_t) = (l + radius, r - radius, b + radius, t - radius);

        // Circle logic for the rounded corners of the rectangle.
        let circle_resolution = corner_resolution * 4;
        let t = 2.0 * std::f64::consts::PI / circle_resolution as Scalar;
        fn f(r: Scalar, i: Scalar, t: Scalar) -> Point {
            [r * (t*i).cos(), r * (t*i).sin()]
        }

        const NUM_CORNERS: usize = 4;
        let points = (0..NUM_CORNERS).flat_map(move |corner| {
            let (in_x, in_y, step) = match corner {
                0 => (in_r, in_t, 0),
                1 => (in_l, in_t, corner_resolution),
                2 => (in_l, in_b, corner_resolution + corner_resolution),
                3 => (in_r, in_b, corner_resolution + corner_resolution + corner_resolution),
                _ => unreachable!(),
            };
            (0..corner_resolution+1).map(move |res| {
                let i = step + res;
                let circle_offset = f(radius, i as f64, t);
                [in_x + circle_offset[0], in_y + circle_offset[1]]
            })
        });

        let (x, y, w, h) = rect.x_y_w_h();
        widget::Polygon::styled(points, *style)
            .x_y(x, y)
            .w_h(w, h)
            .parent(id)
            .graphics_for(id)
            .set(state.ids.polygon, ui);
    }

}

impl Colorable for RoundedRectangle {
    fn color(mut self, color: Color) -> Self {
        self.style.set_color(color);
        self
    }
}
