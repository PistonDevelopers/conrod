
use Scalar;
use color::{Color, Colorable};
use elmesque::Element;
use graphics::character::CharacterCache;
use position::{Point, Rect, Sizeable};
use theme::Theme;
use widget::{self, Widget};


/// A simple, non-interactive widget for drawing a single straight Line.
#[derive(Copy, Clone, Debug)]
pub struct Line {
    /// The start of the line.
    pub start: Point,
    /// The end of the line.
    pub end: Point,
    /// Data necessary and common for all widget types.
    pub common: widget::CommonBuilder,
    /// Unique styling.
    pub style: Style,
}

/// Unique state for the Line widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State {
    /// The start of the line.
    start: Point,
    /// The end of the line.
    end: Point,
}

/// Unique styling for a Line widget.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    /// The patter for the line.
    pub maybe_pattern: Option<Pattern>,
    /// Color of the Button's pressable area.
    pub maybe_color: Option<Color>,
    /// The thickness of the line.
    pub maybe_thickness: Option<Scalar>,
}

/// The pattern used to draw the line.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Pattern {
    Solid,
    Dashed,
    Dotted,
}


impl Line {

    /// Construct a new Line widget builder.
    pub fn new(start: Point, end: Point) -> Self {
        let dim = Rect::from_corners(start, end).dim();
        Line {
            start: start,
            end: end,
            common: widget::CommonBuilder::new(),
            style: Style::new(),
        }.dim(dim)
    }

    /// The thickness or width of the Line.
    ///
    /// Use this instead of `Positionable::width` for the thickness of the `Line`, as `width` and
    /// `height` refer to the dimensions of the bounding rectangle.
    pub fn thickness(mut self, thickness: Scalar) -> Self {
        self.style.maybe_thickness = Some(thickness);
        self
    }

    /// Make a solid line.
    pub fn solid(mut self) -> Self {
        self.style.maybe_pattern = Some(Pattern::Solid);
        self
    }

    /// Make a line with a Dashed pattern.
    pub fn dashed(mut self) -> Self {
        self.style.maybe_pattern = Some(Pattern::Dashed);
        self
    }

    /// Make a line with a Dotted pattern.
    pub fn dotted(mut self) -> Self {
        self.style.maybe_pattern = Some(Pattern::Dotted);
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
        }
    }

    /// The Pattern for the Line.
    pub fn pattern(&self, theme: &Theme) -> Pattern {
        const DEFAULT_PATTERN: Pattern = Pattern::Solid;
        self.maybe_pattern.or_else(|| theme.maybe_line.as_ref().map(|default| {
            default.style.maybe_pattern.unwrap_or(DEFAULT_PATTERN)
        })).unwrap_or(DEFAULT_PATTERN)
    }

    /// The Color for the Line.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or_else(|| theme.maybe_line.as_ref().map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// The width or thickness of the Line.
    pub fn thickness(&self, theme: &Theme) -> Scalar {
        const DEFAULT_THICKNESS: Scalar = 1.0;
        self.maybe_thickness.or_else(|| theme.maybe_line.as_ref().map(|default| {
            default.style.maybe_thickness.unwrap_or(DEFAULT_THICKNESS)
        })).unwrap_or(DEFAULT_THICKNESS)
    }

}


impl Widget for Line {
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> &'static str {
        "Button"
    }

    fn init_state(&self) -> State {
        State {
            start: [0.0, 0.0],
            end: [0.0, 0.0],
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Line.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { state, .. } = args;
        let Line { start, end, .. } = self;

        if state.view().start != start {
            state.update(|state| state.start = start);
        }

        if state.view().end != end {
            state.update(|state| state.end = end);
        }
    }

    /// Construct an Element for the Line.
    fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
        use elmesque::form::{collage, segment, solid, traced};
        let widget::DrawArgs { rect, state, style, theme, .. } = args;
        let (x, y, w, h) = rect.x_y_w_h();
        let color = style.color(theme);
        let thickness = style.thickness(theme);
        let a = (state.start[0], state.start[1]);
        let b = (state.end[0], state.end[1]);
        let form = traced(solid(color).width(thickness), segment(a, b)).shift(x, y);
        collage(w as i32, h as i32, vec![form])
    }

}


impl Colorable for Line {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}


