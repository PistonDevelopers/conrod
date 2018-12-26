//! A simple, non-interactive widget for drawing a series of conjoined lines.

use {Color, Colorable, Point, Positionable, Scalar, Sizeable, Theme, Widget};
use graph;
use utils::{vec2_add, vec2_sub};
use widget;
use widget::triangles::Triangle;

pub use super::line::Pattern;
pub use super::line::Style;


/// A simple, non-interactive widget for drawing a series of lines and/or points.
#[derive(Clone, Debug, WidgetCommon_)]
pub struct PointPath<I> {
    /// Some iterator yielding a series of Points.
    pub points: I,
    /// Data necessary and common for all widget builder types.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// Unique styling for the PointPath.
    pub style: Style,
    /// Whether or not the points should be automatically centred to the widget position.
    pub maybe_shift_to_centre_from: Option<Point>,
}

/// State that is unique to the PointPath.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// An owned version of the list of points.
    pub points: Vec<Point>,
}

/// An iterator that triangulates a point path.
#[derive(Clone)]
pub struct Triangles<I> {
    next: Option<Triangle<Point>>,
    prev: Point,
    points: I,
    half_thickness: Scalar,
    cap: widget::line::Cap,
}


impl<I> PointPath<I> {
    /// The same as [**PointPath::new**](./struct.PointPath#method.new) but with th given style.
    pub fn styled(points: I, style: Style) -> Self {
        PointPath {
            points: points,
            common: widget::CommonBuilder::default(),
            style: style,
            maybe_shift_to_centre_from: None,
        }
    }

    /// Build a new default PointPath widget.
    ///
    /// Note that this does *not* automatically set the position of the bounding box for the
    /// widget. It is recommended that you also see the `abs` and `centred` constructors for smart
    /// positioning and layout that automatically infer the position and size of the bounding box.
    /// This method should only be preferred if the user can also specify the correct bounding box
    /// position and size as this will be more efficient than the `abs` or `centred` methods.
    pub fn new(points: I) -> Self {
        PointPath::styled(points, Style::new())
    }

    /// Build a new PointPath whose bounding box is fit to the absolute co-ordinates of the points.
    ///
    /// This requires that the `points` iterator is `Clone` so that we may iterate through and
    /// determine the bounding box of the `points`.
    ///
    /// If you would rather centre the points to the middle of the bounding box, use
    /// [**PointPath::centred**](./struct.PointPath#method.centred) instead.
    pub fn abs(points: I) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        PointPath::abs_styled(points, Style::new())
    }

    /// The same as [**PointPath::abs**](./struct.PointPath#method.abs) but constructs the
    /// **PointPath** with the given style.
    pub fn abs_styled(points: I, style: Style) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        let points_clone = points.clone().into_iter();
        let (xy, dim) = super::bounding_box_for_points(points_clone).xy_dim();
        PointPath::styled(points, style).wh(dim).xy(xy)
    }

    /// Build a new **PointPath** and shift the location of the points so that the centre of their
    /// bounding rectangle lies at the position determined for the **PointPath** widget.
    ///
    /// This is useful if your points simply describe a shape and you want to position them using
    /// conrod's auto-layout or **Positionable** methods.
    ///
    /// If you would rather centre the bounding box to the points, use
    /// [**PointPath::abs**](./struct.PointPath#method.abs) instead.
    pub fn centred(points: I) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        PointPath::centred_styled(points, Style::new())
    }

    /// The same as [**PointPath::centred**](./struct.PointPath#method.centred) but constructs the
    /// **PointPath** with the given style.
    pub fn centred_styled(points: I, style: Style) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        let points_clone = points.clone().into_iter();
        let (xy, dim) = super::bounding_box_for_points(points_clone).xy_dim();
        let mut point_path = PointPath::styled(points, style).wh(dim);
        point_path.maybe_shift_to_centre_from = Some(xy);
        point_path
    }

    /// The thickness or width of the **PointPath**'s lines.
    ///
    /// Use this instead of `Positionable::width` for the thickness of the `Line`, as `width` and
    /// `height` refer to the dimensions of the bounding rectangle.
    pub fn thickness(mut self, thickness: Scalar) -> Self {
        self.style.set_thickness(thickness);
        self
    }

    /// Make a Solid line.
    pub fn solid(mut self) -> Self {
        self.style.set_pattern(Pattern::Solid);
        self
    }

    /// Make a line with a Dashed pattern.
    pub fn dashed(mut self) -> Self {
        self.style.set_pattern(Pattern::Dashed);
        self
    }

    /// Make a line with a Dotted pattern.
    pub fn dotted(mut self) -> Self {
        self.style.set_pattern(Pattern::Dotted);
        self
    }
}


impl<I> Widget for PointPath<I>
    where I: IntoIterator<Item=Point>,
{
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            points: Vec::new(),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn is_over(&self) -> widget::IsOverFn {
        is_over_widget
    }

    /// Update the state of the Line.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use utils::{iter_diff, IterDiff};
        let widget::UpdateArgs { rect, state, .. } = args;
        let PointPath { points, maybe_shift_to_centre_from, .. } = self;

        // A function that compares the given points iterator to the points currently owned by
        // `State` and updates only if necessary.
        fn update_points<I>(state: &mut widget::State<State>, points: I)
            where I: IntoIterator<Item=Point>,
        {
            match iter_diff(&state.points, points) {
                Some(IterDiff::FirstMismatch(i, mismatch)) => state.update(|state| {
                    state.points.truncate(i);
                    state.points.extend(mismatch);
                }),
                Some(IterDiff::Longer(remaining)) =>
                    state.update(|state| state.points.extend(remaining)),
                Some(IterDiff::Shorter(total)) =>
                    state.update(|state| state.points.truncate(total)),
                None => (),
            }
        }

        match maybe_shift_to_centre_from {
            Some(original) => {
                let xy = rect.xy();
                let difference = vec2_sub(xy, original);
                update_points(state, points.into_iter().map(|point| vec2_add(point, difference)))
            },
            None => update_points(state, points),
        }
    }

}

impl<I> Colorable for PointPath<I> {
    fn color(mut self, color: Color) -> Self {
        self.style.set_color(color);
        self
    }
}


/// Triangulate a point path.
///
/// Returns `None` if the given iterator yields less than one point.
pub fn triangles<I>(points: I, cap: widget::line::Cap, thickness: Scalar)
    -> Option<Triangles<I::IntoIter>>
    where I: IntoIterator<Item=Point>,
{
    let mut points = points.into_iter();
    let first = match points.next() {
        Some(point) => point,
        None => return None,
    };
    Some(Triangles {
        next: None,
        prev: first,
        points: points,
        half_thickness: thickness / 2.0,
        cap: cap,
    })
}

impl<I> Iterator for Triangles<I>
    where I: Iterator<Item=Point>,
{
    type Item = Triangle<Point>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(triangle) = self.next.take() {
            return Some(triangle);
        }
        self.points.next().map(|point| {
            let (a, b) = (self.prev, point);
            self.prev = point;
            let tris = widget::line::triangles(a, b, self.half_thickness);
            self.next = Some(tris[1]);
            tris[0]
        })
    }
}

/// Returns whether or not the given point `p` lies over the `PointPath` described by the given
/// points, line cap and thickness.
pub fn is_over<I>(points: I, cap: widget::line::Cap, thickness: Scalar, p: Point) -> bool
where
    I: IntoIterator<Item=Point>,
{
    triangles(points, cap, thickness).map(|ts| widget::triangles::is_over(ts, p)).unwrap_or(false)
}

/// The function to use for picking whether a given point is over the point path.
pub fn is_over_widget(widget: &graph::Container, point: Point, theme: &Theme) -> widget::IsOver {
    widget
        .state_and_style::<State, Style>()
        .map(|widget| {
            let cap = widget.style.get_cap(theme);
            let thickness = widget.style.get_thickness(theme);
            is_over(widget.state.points.iter().cloned(), cap, thickness, point)
        })
        .unwrap_or_else(|| widget.rect.is_over(point))
        .into()
}
