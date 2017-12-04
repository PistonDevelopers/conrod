//! A simple, non-interactive widget for drawing a single **Oval**.

use {Color, Colorable, Dimensions, Point, Rect, Scalar, Sizeable, Widget};
use std;
use super::Style as Style;
use widget;
use widget::triangles::Triangle;


/// A simple, non-interactive widget for drawing a single **Oval**.
#[derive(Copy, Clone, Debug, WidgetCommon_)]
pub struct Oval<S> {
    /// Data necessary and common for all widget builder types.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// Unique styling.
    pub style: Style,
    /// The number of lines used to draw the edge.
    pub resolution: usize,
    /// A type describing the section of the `Oval` that is to be drawn.
    pub section: S,
}

/// Types that may be used to describe the visible section of the `Oval`.
pub trait OvalSection: 'static + Copy + PartialEq + Send {}

/// The entire `Oval` will be drawn.
///
/// To draw only a section of the oval, use the `section` builder method.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Full;

impl OvalSection for Full {}

/// A section of the oval will be drawn where the section is specified by the given radians.
///
/// A section with `radians` of `2.0 * PI` would be equivalent to the full `Oval`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Section {
    /// The angle occuppied by the section's circumference.
    pub radians: Scalar,
    /// The radians at which the section will begin.
    ///
    /// A value of `0.0` will begin at the right of the oval.
    pub offset_radians: Scalar,
}

impl OvalSection for Section {}

/// Unique state for the **Oval**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State<S> {
    /// The number of lines used to draw the edge.
    pub resolution: usize,
    /// A type describing the section of the `Oval` that is to be drawn.
    pub section: S,
}

/// The default circle resolution if none is specified.
pub const DEFAULT_RESOLUTION: usize = 50;

impl Oval<Full> {
    /// Build an **Oval** with the given dimensions and style.
    pub fn styled(dim: Dimensions, style: Style) -> Self {
        Oval {
            common: widget::CommonBuilder::default(),
            style: style,
            resolution: DEFAULT_RESOLUTION,
            section: Full,
        }.wh(dim)
    }

    /// Build a new **Fill**ed **Oval**.
    pub fn fill(dim: Dimensions) -> Self {
        Oval::styled(dim, Style::fill())
    }

    /// Build a new **Oval** **Fill**ed with the given color.
    pub fn fill_with(dim: Dimensions, color: Color) -> Self {
        Oval::styled(dim, Style::fill_with(color))
    }

    /// Build a new **Outline**d **Oval** widget.
    pub fn outline(dim: Dimensions) -> Self {
        Oval::styled(dim, Style::outline())
    }

    /// Build a new **Oval** **Outline**d with the given style.
    pub fn outline_styled(dim: Dimensions, line_style: widget::line::Style) -> Self {
        Oval::styled(dim, Style::outline_styled(line_style))
    }
}

impl<S> Oval<S> {
    /// The number of lines used to draw the edge.
    ///
    /// By default, `DEFAULT_RESOLUTION` is used.
    pub fn resolution(mut self, resolution: usize) -> Self {
        self.resolution = resolution;
        self
    }

    /// Produces an `Oval` where only a section is drawn.
    ///
    /// The given `radians` describes the angle occuppied by the section's circumference.
    pub fn section(self, radians: Scalar) -> Oval<Section> {
        let Oval { common, style, resolution, .. } = self;
        let section = Section { radians, offset_radians: 0.0 };
        Oval { common, style, resolution, section }
    }
}

impl Oval<Section> {
    /// The radians at which the section will begin.
    ///
    /// A value of `0.0` will begin at the rightmost point of the oval.
    pub fn offset_radians(mut self, offset_radians: Scalar) -> Self {
        self.section.offset_radians = offset_radians;
        self
    }
}

impl<S> Widget for Oval<S>
where
    S: OvalSection,
{
    type State = State<S>;
    type Style = Style;
    type Event = ();

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            resolution: DEFAULT_RESOLUTION,
            section: self.section,
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, .. } = args;
        if state.resolution != self.resolution {
            state.update(|state| state.resolution = self.resolution);
        }
    }
}

impl<S> Colorable for Oval<S> {
    fn color(mut self, color: Color) -> Self {
        self.style.set_color(color);
        self
    }
}

/// An iterator yielding the `Oval`'s edges as a circumference represented as a series of points.
///
/// `resolution` is clamped to a minimum of `1` as to avoid creating a `Circumference` that
/// produces `NaN` values.
pub fn circumference(rect: Rect, mut resolution: usize) -> Circumference {
    resolution = std::cmp::max(resolution, 1);
    use std::f64::consts::PI;
    let (x, y, w, h) = rect.x_y_w_h();
    Circumference {
        index: 0,
        num_points: resolution + 1,
        point: [x, y],
        half_w: w / 2.0,
        half_h: h / 2.0,
        rad_step: 2.0 * PI / resolution as Scalar,
        rad_offset: 0.0,
    }
}

/// An iterator yielding the triangles that describe the given oval.
pub fn triangles(rect: Rect, resolution: usize) -> Triangles {
    circumference(rect, resolution).triangles()
}

/// An iterator yielding the edges of an `Oval` (or some section of an `Oval`) as a circumference
/// represented as a series of edges.
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct Circumference {
    index: usize,
    num_points: usize,
    point: Point,
    rad_step: Scalar,
    rad_offset: Scalar,
    half_w: Scalar,
    half_h: Scalar,
}

/// An iterator yielding triangles that describe an oval or some section of an oval.
#[derive(Clone)]
pub struct Triangles {
    // The last circumference point yielded by the `CircumferenceOffset` iterator.
    last: Point,
    // The circumference points used to yield yielded by the `CircumferenceOffset` iterator.
    points: Circumference,
}

impl Circumference {
    /// Produces a new iterator that yields only a section of the `Oval`'s circumference, where the
    /// section is described via its angle in radians.
    pub fn section(mut self, radians: Scalar) -> Self {
        let resolution = self.num_points - 1;
        self.rad_step = radians / resolution as Scalar;
        self
    }

    /// Rotates the position at which the iterator starts yielding points by the given radians.
    ///
    /// This is particularly useful for yielding a different section of the circumference when
    /// using `circumference_section`
    pub fn offset_radians(mut self, radians: Scalar) -> Self {
        self.rad_offset = radians;
        self
    }

    /// Produces an `Iterator` yielding `Triangle`s.
    ///
    /// Triangles are created by joining each edge yielded by the inner `Circumference` to the
    /// middle of the `Oval`.
    pub fn triangles(mut self) -> Triangles {
        let last = self.next().unwrap_or(self.point);
        Triangles { last, points: self }
    }
}

impl Iterator for Circumference {
    type Item = Point;
    fn next(&mut self) -> Option<Self::Item> {
        let Circumference {
            ref mut index,
            num_points,
            point,
            rad_step,
            rad_offset,
            half_w,
            half_h,
        } = *self;
        if *index >= num_points {
            return None;
        }
        let x = point[0] + half_w * (rad_offset + rad_step * *index as Scalar).cos();
        let y = point[1] + half_h * (rad_offset + rad_step * *index as Scalar).sin();
        *index += 1;
        Some([x, y])
    }
}

impl Iterator for Triangles {
    type Item = Triangle<Point>;
    fn next(&mut self) -> Option<Self::Item> {
        let Triangles { ref mut points, ref mut last } = *self;
        points.next().map(|next| {
            let triangle = Triangle([points.point, *last, next]);
            *last = next;
            triangle
        })
    }
}
