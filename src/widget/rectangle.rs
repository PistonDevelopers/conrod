
use Scalar;
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use graphics::character::CharacterCache;
use label::{FontSize, Labelable};
use mouse::Mouse;
use position::Positionable;
use theme::Theme;
use ui::GlyphCache;
use widget::{self, Widget};


/// A basic, non-interactive rectangle shape widget.
#[derive(Clone, Debug)]
pub struct Rectangle {
    common: widget::CommonBuilder,
    style: Style,
}

/// Styling for the Rectangle.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Style {
    /// The outline of the rectangle with this style.
    Outline(super::line::Style),
    /// A rectangle filled with this color.
    Filled(Option<Color>),
}

impl Style {

    /// The default Rectangle Style.
    pub fn new() -> Self {
        Style::Filled(None)
    }

    /// Get the color of the Rectangle.
    pub fn get_color(&self, theme: &Theme) -> Color {
        match *self {
            Style::Filled(maybe_color) => maybe_color.unwrap_or(theme.shape_color),
            Style::Outline(style) => style.get_color(theme),
        }
    }

}

impl Rectangle {

    /// Create a new rectangle builder.
    pub fn new() -> Self {
        Rectangle {
            common: widget::CommonBuilder::new(),
            style: Style::new(),
        }
    }

    /// Build an outlined rectangle rather than a filled one.
    pub fn outline(mut self, mut line_style: super::line::Style) -> Self {
        if line_style.maybe_color.is_none() {
            line_style.maybe_color = match self.style {
                Style::Outline(prev_style) => prev_style.maybe_color,
                Style::Filled(maybe_color) => maybe_color,
            };
        }
        self.style = Style::Outline(line_style);
        self
    }

}


impl Widget for Rectangle {
    type State = ();
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> &'static str {
        "Rectangle"
    }

    fn init_state(&self) -> () {
        ()
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Rectangle.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        // No unique state to update for the boring ol' Rectangle!
    }

    /// Construct an Element for the Line.
    fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
        use elmesque::form::{self, collage, segment, solid, traced};
        let widget::DrawArgs { rect, state, style, theme, .. } = args;
        let (x, y, w, h) = rect.x_y_w_h();

        let color = style.get_color(theme);
        match *style {
            Style::Filled(_) => {
                let form = form::rect(w, h).filled(color).shift(x, y);
                collage(w as i32, h as i32, vec![form])
            },
            Style::Outline(line_style) => {
                let thickness = line_style.get_thickness(theme);
                let a = (rect.x.start, rect.y.end);
                let b = (rect.x.end, rect.y.end);
                let c = (rect.x.end, rect.y.start);
                let d = (rect.x.start, rect.y.start);
                // TODO: Use PointPath
                ::elmesque::element::empty()
            },
        }
    }


}

