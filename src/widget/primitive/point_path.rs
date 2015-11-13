use CharacterCache;
use elmesque::Element;
use position::{Point, Positionable, Range, Rect, Sizeable};
use super::line::Style as LineStyle;
use theme::Theme;
use vecmath::{vec2_add, vec2_sub};
use widget::{self, Widget};


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
    points: Vec<Point>,
}

/// Styling that is unique to the PointPath.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    /// Whether or not to draw Lines, Points or Both.
    pub maybe_kind: Option<StyleKind>,
}

/// Whether or not to draw Lines, Points or Both.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum StyleKind {
    /// Draw only the lines between the points.
    Lines(LineStyle),
    /// Draw only the points.
    Points,
    /// Draw both the lines and the points.
    Both(LineStyle),
}

// pub struct PointStyle {
//     shape: 
// }


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
        PointPath::styled(points, style).dim(dim).point(xy)
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
        let mut point_path = PointPath::styled(points, style).dim(dim);
        point_path.maybe_shift_to_centre_from = Some(xy);
        point_path
    }

}


impl Style {

    /// Constructor for a default **PointPath** Style.
    pub fn new() -> Self {
        Style {
            maybe_kind: None,
        }
    }

    /// Constructor for a specific kind of **PointPath** Style.
    pub fn from_kind(kind: StyleKind) -> Self {
        Style {
            maybe_kind: Some(kind),
        }
    }

    /// Construct a **PointPath** drawn as the lines between the points.
    pub fn lines() -> Self {
        Style::lines_styled(LineStyle::new())
    }

    /// Construct a **PointPath** with only the points drawn.
    pub fn points() -> Self {
        Style::from_kind(StyleKind::Points)
    }

    /// Construct a **PointPath** with both lines and points drawn.
    pub fn lines_and_points() -> Self {
        Style::from_kind(StyleKind::Both(LineStyle::new()))
    }

    /// Same as [**Style::lines**](./struct.Style#method.lines) but with the given style.
    pub fn lines_styled(style: LineStyle) -> Self {
        Style::from_kind(StyleKind::Lines(style))
    }

    /// Set the kind of styling for the **PointPath**.
    pub fn set_kind(&mut self, kind: StyleKind) {
        self.maybe_kind = Some(kind);
    }

    /// Get the kind of styling for the **PointPath** Style.
    pub fn get_kind(&self, theme: &Theme) -> StyleKind {
        fn default_kind() -> StyleKind {
            StyleKind::Lines(super::line::Style::new())
        }
        self.maybe_kind.or_else(|| theme.maybe_point_path.as_ref().map(|default| {
            default.style.maybe_kind.unwrap_or_else(default_kind)
        })).unwrap_or_else(default_kind)
    }

}


impl<I> Widget for PointPath<I>
    where I: IntoIterator<Item=Point>,
{
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> &'static str {
        "Line"
    }

    fn init_state(&self) -> State {
        State {
            points: Vec::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Line.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        use utils::{iter_diff, IterDiff};
        let widget::UpdateArgs { rect, state, .. } = args;
        let PointPath { points, maybe_shift_to_centre_from, .. } = self;

        // A function that compares the given points iterator to the points currently owned by
        // `State` and updates only if necessary.
        fn update_points<I>(state: &mut widget::State<State>, points: I)
            where I: IntoIterator<Item=Point>,
        {
            match iter_diff(&state.view().points, points) {
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

    /// Construct an Element for the Line.
    fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
        let widget::DrawArgs { rect, state, style, theme, .. } = args;
        match style.get_kind(theme) {
            StyleKind::Lines(line_style) | StyleKind::Both(line_style) => {
                draw_lines(state.points.iter().cloned(), rect, line_style, theme)
            },
            StyleKind::Points => unimplemented!(),
        }
    }
}


/// Produce a renderable **Element** for the given point path as a series of lines.
pub fn draw_lines<I>(points: I, rect: Rect, style: LineStyle, theme: &Theme) -> Element
    where I: Iterator<Item=Point> + Clone,
{
    use elmesque::form::{collage, segment, solid, traced};
    let mut ends = points.clone();
    ends.next().map(|_| {
        let (w, h) = rect.w_h();
        let color = style.get_color(theme);
        let thickness = style.get_thickness(theme);
        let forms = points.zip(ends).map(move |(start, end)| {
            let a = (start[0], start[1]);
            let b = (end[0], end[1]);
            traced(solid(color).width(thickness), segment(a, b))
        });
        collage(w as i32, h as i32, forms.collect())
    }).unwrap_or_else(|| ::elmesque::element::empty())
}
