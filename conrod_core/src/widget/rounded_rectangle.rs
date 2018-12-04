//! Similar to the `Rectangle` widget however is drawn with rounded corners.
//!
//! The roundedness of the corners is specified with a `radius`. This indicates the radius of the
//! circle used to draw the corners.

use {Color, Colorable, Dimensions, Point, Positionable, Range, Rect, Scalar, Sizeable, Theme,
     Widget};
use graph;
use std::f64::consts::PI;
use widget;
use widget::primitive::shape::Style;
use widget::primitive::shape::oval::Circumference;


/// Draws a rectangle with corners rounded via the given radius.
#[derive(Copy, Clone, Debug, WidgetCommon_)]
pub struct RoundedRectangle {
    /// Data necessary and common for all widget builder types.
    #[conrod(common_builder)]
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
            common: widget::CommonBuilder::default(),
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

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn is_over(&self) -> widget::IsOverFn {
        is_over_widget
    }

    /// Update the state of the Rectangle.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, style, rect, ui, .. } = args;
        let RoundedRectangle { radius, corner_resolution, .. } = self;
        let points = points(rect, radius, corner_resolution);
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

/// An iterator yielding the outer points of a `RoundedRectangle`
#[derive(Clone)]
pub struct Points {
    rect: Rect,
    corner_rect: Rect,
    corner_index: usize,
    corner_resolution: usize,
    corner_points: Circumference,
}

const CORNER_RADIANS: Scalar = PI * 0.5;

/// Produce an iterator yielding the outer points of a rounded rectangle.
///
/// - `rect` describes the location and dimensions
/// - `radius` describes the radius of the corner circles.
/// - `corner_resolution` the number of lines used to draw each corner.
pub fn points(rect: Rect, radius: Scalar, corner_resolution: usize) -> Points {
    let (r, t) = (rect.x.end, rect.y.end);
    // First corner is the top right corner.
    let radius_2 = radius * 2.0;
    let corner_rect = Rect {
        x: Range { start: r - radius_2, end: r },
        y: Range { start: t - radius_2, end: t },
    };
    let corner = Circumference::new_section(corner_rect, corner_resolution, CORNER_RADIANS);
    Points {
        rect,
        corner_rect,
        corner_index: 0,
        corner_resolution,
        corner_points: corner,
    }
}

impl Iterator for Points {
    type Item = Point;
    fn next(&mut self) -> Option<Self::Item> {
        let Points {
            ref mut corner_rect,
            ref mut corner_index,
            ref mut corner_points,
            corner_resolution,
            rect,
        } = *self;
        loop {
            if let Some(point) = corner_points.next() {
                return Some(point);
            }
            *corner_rect = match *corner_index {
                0 => corner_rect.align_left_of(rect),
                1 => corner_rect.align_bottom_of(rect),
                2 => corner_rect.align_right_of(rect),
                _ => return None,
            };
            *corner_index += 1;
            let offset_radians = *corner_index as Scalar * CORNER_RADIANS;
            *corner_points = Circumference::new_section(*corner_rect, corner_resolution, CORNER_RADIANS)
                .offset_radians(offset_radians);
        }
    }
}

/// An iterator yielding triangles for a `RoundedRectangle`.
pub type Triangles = widget::polygon::Triangles<Points>;

/// The function to use for picking whether a given point is over the polygon.
pub fn is_over_widget(widget: &graph::Container, point: Point, _: &Theme) -> widget::IsOver {
    widget
        .unique_widget_state::<RoundedRectangle>()
        .map(|widget| widget.state.ids.polygon.into())
        .unwrap_or_else(|| widget.rect.is_over(point).into())
}
