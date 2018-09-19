//! A simple, non-interactive widget for drawing a single straight Line.

use {Color, Colorable, Point, Positionable, Rect, Scalar, Sizeable, Theme};
use graph;
use utils::{vec2_add, vec2_sub};
use widget::{self, Widget};
use widget::triangles::Triangle;


/// A simple, non-interactive widget for drawing a single straight Line.
#[derive(Copy, Clone, Debug, WidgetCommon_)]
pub struct Line {
    /// Data necessary and common for all widget builder types.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// The start of the line.
    pub start: Point,
    /// The end of the line.
    pub end: Point,
    /// Unique styling.
    pub style: Style,
    /// Whether or not the line should be automatically centred to the widget position.
    pub should_centre_points: bool,
}

/// Unique state for the Line widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State {
    /// The start of the line.
    pub start: Point,
    /// The end of the line.
    pub end: Point,
}

/// Unique styling for a Line widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    /// The patter for the line.
    pub maybe_pattern: Option<Pattern>,
    /// Color of the Button's pressable area.
    pub maybe_color: Option<Color>,
    /// The thickness of the line.
    pub maybe_thickness: Option<Scalar>,
    /// The style with which the ends of the line are drawn.
    pub maybe_cap: Option<Cap>,
}

/// The pattern used to draw the line.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Pattern {
    /// A single continuous stroke.
    Solid,
    /// A series of line strokes.
    Dashed,
    /// A series of circles.
    Dotted,
}

/// Whether the end of the **Line** should be flat or rounded.
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cap {
    /// The line is capped with a flat edge.
    Flat,
    /// The line is capped with a semi-circle.
    Round,
}


impl Line {

    /// Build a new **Line** widget with the given style.
    pub fn styled(start: Point, end: Point, style: Style) -> Self {
        Line {
            start: start,
            end: end,
            common: widget::CommonBuilder::default(),
            style: style,
            should_centre_points: false,
        }
    }

    /// Build a new default **Line** widget.
    pub fn new(start: Point, end: Point) -> Self {
        Line::styled(start, end, Style::new())
    }

    /// Build a new **Line** whose bounding box is fit to the absolute co-ordinates of the line
    /// points.
    ///
    /// If you would rather centre the start and end to the middle of the bounding box, use
    /// [**Line::centred**](./struct.Line#method.centred) instead.
    pub fn abs(start: Point, end: Point) -> Self {
        Line::abs_styled(start, end, Style::new())
    }

    /// The same as [**Line::abs**](./struct.Line#method.abs) but with the given style.
    pub fn abs_styled(start: Point, end: Point, style: Style) -> Self {
        let (xy, dim) = Rect::from_corners(start, end).xy_dim();
        Line::styled(start, end, style).wh(dim).xy(xy)
    }

    /// Build a new **Line** and shift the location of the start and end points so that the centre
    /// of their bounding rectangle lies at the position determined by the layout for the **Line**
    /// widget.
    ///
    /// This is useful if your points simply describe the line's angle and magnitude, and you want
    /// to position them using conrod's auto-layout or `Positionable` methods.
    ///
    /// If you would rather centre the bounding box to the points, use
    /// [**Line::abs**](./struct.Line#method.abs) instead.
    pub fn centred(start: Point, end: Point) -> Self {
        Line::centred_styled(start, end, Style::new())
    }

    /// The same as [**Line::centred**](./struct.Line#method.centred) but with the given style.
    pub fn centred_styled(start: Point, end: Point, style: Style) -> Self {
        let dim = Rect::from_corners(start, end).dim();
        let mut line = Line::styled(start, end, style).wh(dim);
        line.should_centre_points = true;
        line
    }

    /// The thickness or width of the Line.
    ///
    /// Use this instead of `Positionable::width` for the thickness of the `Line`, as `width` and
    /// `height` refer to the dimensions of the bounding rectangle.
    pub fn thickness(mut self, thickness: Scalar) -> Self {
        self.style.set_thickness(thickness);
        self
    }

    /// Make a solid line.
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


impl Style {

    /// Constructor for a default Line Style.
    pub fn new() -> Self {
        Style {
            maybe_pattern: None,
            maybe_color: None,
            maybe_thickness: None,
            maybe_cap: None,
        }
    }

    /// Make a solid line.
    pub fn solid() -> Self {
        Style::new().pattern(Pattern::Solid)
    }

    /// Make a line with a Dashed pattern.
    pub fn dashed() -> Self {
        Style::new().pattern(Pattern::Dashed)
    }

    /// Make a line with a Dotted pattern.
    pub fn dotted() -> Self {
        Style::new().pattern(Pattern::Dotted)
    }

    /// The style with some given pattern.
    pub fn pattern(mut self, pattern: Pattern) -> Self {
        self.set_pattern(pattern);
        self
    }

    /// The style with some given color.
    pub fn color(mut self, color: Color) -> Self {
        self.set_color(color);
        self
    }

    /// The style with some given thickness.
    pub fn thickness(mut self, thickness: Scalar) -> Self {
        self.set_thickness(thickness);
        self
    }

    /// The style for the ends of the Line.
    pub fn cap(mut self, cap: Cap) -> Self {
        self.set_cap(cap);
        self
    }

    /// Set the pattern for the line.
    pub fn set_pattern(&mut self, pattern: Pattern) {
        self.maybe_pattern = Some(pattern);
    }

    /// Set the color for the line.
    pub fn set_color(&mut self, color: Color) {
        self.maybe_color = Some(color);
    }

    /// Set the thickness for the line.
    pub fn set_thickness(&mut self, thickness: Scalar) {
        self.maybe_thickness = Some(thickness);
    }

    /// Set the **Cap** for the line.
    pub fn set_cap(&mut self, cap: Cap) {
        self.maybe_cap = Some(cap);
    }

    /// The Pattern for the Line.
    pub fn get_pattern(&self, theme: &Theme) -> Pattern {
        const DEFAULT_PATTERN: Pattern = Pattern::Solid;
        self.maybe_pattern.or_else(|| theme.widget_style::<Style>().map(|default| {
            default.style.maybe_pattern.unwrap_or(DEFAULT_PATTERN)
        })).unwrap_or(DEFAULT_PATTERN)
    }

    /// The Color for the Line.
    pub fn get_color(&self, theme: &Theme) -> Color {
        self.maybe_color.or_else(|| theme.widget_style::<Style>().map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// The width or thickness of the Line.
    pub fn get_thickness(&self, theme: &Theme) -> Scalar {
        const DEFAULT_THICKNESS: Scalar = 1.0;
        self.maybe_thickness.or_else(|| theme.widget_style::<Style>().map(|default| {
            default.style.maybe_thickness.unwrap_or(DEFAULT_THICKNESS)
        })).unwrap_or(DEFAULT_THICKNESS)
    }

    /// The styling for the ends of the Line.
    pub fn get_cap(&self, theme: &Theme) -> Cap {
        const DEFAULT_CAP: Cap = Cap::Flat;
        self.maybe_cap.or_else(|| theme.widget_style::<Style>().map(|default| {
            default.style.maybe_cap.unwrap_or(DEFAULT_CAP)
        })).unwrap_or(DEFAULT_CAP)
    }

}


impl Widget for Line {
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            start: [0.0, 0.0],
            end: [0.0, 0.0],
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
        let widget::UpdateArgs { rect, state, .. } = args;
        let Line { mut start, mut end, should_centre_points, .. } = self;

        // Check whether or not we need to shift the line to the xy position.
        if should_centre_points {
            let original = Rect::from_corners(start, end).xy();
            let xy = rect.xy();
            let difference = vec2_sub(xy, original);
            start = vec2_add(start, difference);
            end = vec2_add(end, difference);
        }

        if state.start != start {
            state.update(|state| state.start = start);
        }

        if state.end != end {
            state.update(|state| state.end = end);
        }
    }
}


impl Colorable for Line {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

/// Given two points and half the line thickness, return the four corners of the rectangle
/// describing the line.
pub fn rect_corners(a: Point, b: Point, half_thickness: Scalar) -> [Point; 4] {
    let direction = [b[0] - a[0], b[1] - a[1]];
    let mag = (direction[0] * direction[0] + direction[1] * direction[1]).sqrt();
    let unit = [direction[0] / mag, direction[1] / mag];
    let normal = [-unit[1], unit[0]];
    let n = [normal[0] * half_thickness, normal[1] * half_thickness];
    let r1 = [a[0] + n[0], a[1] + n[1]];
    let r2 = [a[0] - n[0], a[1] - n[1]];
    let r3 = [b[0] + n[0], b[1] + n[1]];
    let r4 = [b[0] - n[0], b[1] - n[1]];
    [r1, r2, r3, r4]
}

/// Given two points and half the line thickness, return the two triangles that describe the line.
pub fn triangles(a: Point, b: Point, half_thickness: Scalar) -> [Triangle<Point>; 2] {
    let r = rect_corners(a, b, half_thickness);
    let t1 = Triangle([r[0], r[3], r[1]]);
    let t2 = Triangle([r[0], r[3], r[2]]);
    [t1, t2]
}

/// Describes whether or not the given point touches the line described by *a -> b* with the given
/// thickness.
pub fn is_over(a: Point, b: Point, thickness: Scalar, point: Point) -> bool {
    let half_thickness = thickness * 0.5;
    let tris = triangles(a, b, half_thickness);
    widget::triangles::is_over(tris.iter().cloned(), point)
}

/// The function to use for picking whether a given point is over the line.
pub fn is_over_widget(widget: &graph::Container, point: Point, theme: &Theme) -> widget::IsOver {
    widget
        .unique_widget_state::<Line>()
        .map(|widget| {
            let thickness = widget.style.get_thickness(theme);
            let (a, b) = (widget.state.start, widget.state.end);
            is_over(a, b, thickness, point)
        })
        .unwrap_or_else(|| widget.rect.is_over(point))
        .into()
}
