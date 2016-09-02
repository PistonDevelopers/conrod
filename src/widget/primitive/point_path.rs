//! A simple, non-interactive widget for drawing a series of conjoined lines.

use {
    Color,
    Colorable,
    Point,
    Positionable,
    Range,
    Rect,
    Scalar,
    Sizeable,
    Widget,
};
use utils::{vec2_add, vec2_sub};
use widget;

pub use super::line::Pattern;
pub use super::line::Style;


/// A simple, non-interactive widget for drawing a series of lines and/or points.
#[derive(Clone, Debug)]
pub struct PointPath<I> {
    /// Some iterator yielding a series of Points.
    pub points: I,
    /// Data necessary and common for all widget builder types.
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


/// Find the bounding rect for the given series of points.
fn bounding_box_for_points<I>(mut points: I) -> Rect
    where I: Iterator<Item=Point>,
{
    points.next().map(|first| {
        let start_rect = Rect {
            x: Range { start: first[0], end: first[0] },
            y: Range { start: first[1], end: first[1] },
        };
        points.fold(start_rect, Rect::stretch_to_point)
    }).unwrap_or_else(|| Rect::from_xy_dim([0.0, 0.0], [0.0, 0.0]))
}


impl<I> PointPath<I> {

    /// The same as [**PointPath::new**](./struct.PointPath#method.new) but with th given style.
    pub fn styled(points: I, style: Style) -> Self {
        PointPath {
            points: points,
            common: widget::CommonBuilder::new(),
            style: style,
            maybe_shift_to_centre_from: None,
        }
    }

    /// Build a new default PointPath widget.
    ///
    /// It is recommended that you also see the `abs` and `centred` constructors for smart
    /// positioning and layout.
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
        let (xy, dim) = bounding_box_for_points(points_clone).xy_dim();
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
        let (xy, dim) = bounding_box_for_points(points_clone).xy_dim();
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

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            points: Vec::new(),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
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
