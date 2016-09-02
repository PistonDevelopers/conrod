//! A simple, non-interactive **Polygon** widget for drawing arbitrary convex shapes.

use {Color, Colorable, Point, Positionable, Sizeable, Widget};
use super::Style;
use widget;
use utils::{bounding_box_for_points, vec2_add, vec2_sub};


/// A basic, non-interactive, arbitrary **Polygon** widget.
///
/// The **Polygon** is described by specifying its corners in order.
///
/// **Polygon** will automatically close all shapes, so the given list of points does not need to
/// start and end with the same position.
#[derive(Copy, Clone, Debug)]
pub struct Polygon<I> {
    /// The points describing the corners of the **Polygon**.
    pub points: I,
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// Unique styling for the **Polygon**.
    pub style: Style,
    /// Whether or not the points should be automatically centred to the widget position.
    pub maybe_shift_to_centre_from: Option<Point>,
}

/// Unique state for the **Polygon**.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// Whether the rectangle is drawn as an outline or a filled color.
    kind: Kind,
    /// An owned version of the points yielded by the **Polygon**'s `points` iterator.
    pub points: Vec<Point>,
}

/// Whether the rectangle is drawn as an outline or a filled color.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Kind {
    /// Only the outline of the rectangle is drawn.
    Outline,
    /// The rectangle area is filled with some color.
    Fill,
}


impl<I> Polygon<I> {

    /// Build a polygon with the given points and style.
    pub fn styled(points: I, style: Style) -> Self {
        Polygon {
            points: points,
            common: widget::CommonBuilder::new(),
            style: style,
            maybe_shift_to_centre_from: None,
        }
    }

    /// Build a **Polygon** with the default **Fill** style.
    pub fn fill(points: I) -> Self {
        Polygon::styled(points, Style::fill())
    }

    /// Build a **Polygon** **Fill**ed with the given **Color**.
    pub fn fill_with(points: I, color: Color) -> Self {
        Polygon::styled(points, Style::fill_with(color))
    }

    /// Build a **Polygon** with the default **Outline** style.
    pub fn outline(points: I) -> Self {
        Polygon::styled(points, Style::outline())
    }

    /// Build a **Polygon** **Outline**ed with the given line style.
    pub fn outline_styled(points: I, style: widget::line::Style) -> Self {
        Polygon::styled(points, Style::outline_styled(style))
    }

    /// Build a new filled **Polygon** whose bounding box is fit to the absolute co-ordinates of
    /// the points.
    ///
    /// This requires that the `points` iterator is `Clone` so that we may iterate through and
    /// determine the bounding box of the `points`.
    ///
    /// If you would rather centre the points to the middle of the bounding box, use
    /// the [**Polygon::centred**](./struct.Polygon#method.centred) methods instead.
    pub fn abs_styled(points: I, style: Style) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        let points_clone = points.clone().into_iter();
        let (xy, dim) = bounding_box_for_points(points_clone).xy_dim();
        Polygon::styled(points, style).wh(dim).xy(xy)
    }

    /// The same as [**Polygon::abs_styled**](./struct.Polygon#method.abs_styled) but builds the
    /// **Polygon** with the default **Fill** style.
    pub fn abs_fill(points: I) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::abs_styled(points, Style::fill())
    }

    /// The same as [**Polygon::abs_styled**](./struct.Polygon#method.abs_styled) but builds the
    /// **Polygon** **Fill**ed with the given **Color**.
    pub fn abs_fill_with(points: I, color: Color) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::abs_styled(points, Style::fill_with(color))
    }

    /// The same as [**Polygon::abs_styled**](./struct.Polygon#method.abs_styled) but builds the
    /// **Polygon** with the default **Outline** style.
    pub fn abs_outline(points: I) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::abs_styled(points, Style::outline())
    }

    /// The same as [**Polygon::abs_styled**](./struct.Polygon#method.abs_styled) but builds the
    /// **Polygon** with the given **Outline** styling.
    pub fn abs_outline_styled(points: I, style: widget::line::Style) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::abs_styled(points, Style::outline_styled(style))
    }

    /// Build a new **Polygon** and shift the location of the points so that the centre of their
    /// bounding rectangle lies at the position determined for the **Polygon** widget.
    ///
    /// This is useful if your points simply describe a shape and you want to position them using
    /// conrod's auto-layout and/or **Positionable** methods.
    ///
    /// If you would rather centre the bounding box to the points, use the
    /// [**Polygon::abs**](./struct.Polygon#method.abs) constructor method instead.
    pub fn centred_styled(points: I, style: Style) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        let points_clone = points.clone().into_iter();
        let (xy, dim) = bounding_box_for_points(points_clone).xy_dim();
        let mut polygon = Polygon::styled(points, style).wh(dim);
        polygon.maybe_shift_to_centre_from = Some(xy);
        polygon
    }

    /// The same as [**Polygon::centred_styled**](./struct.Polygon#method.centred_styled) but
    /// constructs the **Polygon** with the default **Fill** style.
    pub fn centred_fill(points: I) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::centred_styled(points, Style::fill())
    }

    /// The same as [**Polygon::centred_styled**](./struct.Polygon#method.centred_styled) but
    /// constructs the **Polygon** **Fill**ed with the given color.
    pub fn centred_fill_with(points: I, color: Color) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::centred_styled(points, Style::fill_with(color))
    }

    /// The same as [**Polygon::centred_styled**](./struct.Polygon#method.centred_styled) but
    /// constructs the **Polygon** with the default **Outline** style.
    pub fn centred_outline(points: I) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::centred_styled(points, Style::outline())
    }

    /// The same as [**Polygon::centred_styled**](./struct.Polygon#method.centred_styled) but
    /// constructs the **Polygon** **Outline**d with the given styling.
    pub fn centred_outline_styled(points: I, style: widget::line::Style) -> Self
        where I: IntoIterator<Item=Point> + Clone,
    {
        Polygon::centred_styled(points, Style::outline_styled(style))
    }

}


impl<I> Widget for Polygon<I>
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
            kind: Kind::Fill,
            points: Vec::new(),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// Update the state of the Polygon.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        use utils::{iter_diff, IterDiff};
        let widget::UpdateArgs { rect, state, style, .. } = args;
        let Polygon { points, maybe_shift_to_centre_from, .. } = self;

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

        // Check whether or not we need to centre the points.
        match maybe_shift_to_centre_from {
            Some(original) => {
                let xy = rect.xy();
                let difference = vec2_sub(xy, original);
                update_points(state, points.into_iter().map(|point| vec2_add(point, difference)))
            },
            None => update_points(state, points),
        }

        let kind = match *style {
            Style::Fill(_) => Kind::Fill,
            Style::Outline(_) => Kind::Outline,
        };

        if state.kind != kind {
            state.update(|state| state.kind = kind);
        }
    }

}


impl<I> Colorable for Polygon<I> {
    fn color(mut self, color: Color) -> Self {
        self.style.set_color(color);
        self
    }
}
