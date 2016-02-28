use {
    Backend,
    Color,
    Colorable,
    Dimensions,
    LineStyle,
    Sizeable,
    Widget,
};
use super::Style as Style;
use widget;


/// A basic, non-interactive rectangle shape widget.
#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// Unique styling for the **Rectangle**.
    pub style: Style,
}

/// Unique state for the Rectangle.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State {
    kind: Kind,
}

/// Whether the rectangle is drawn as an outline or a filled color.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Kind {
    /// Only the outline of the rectangle is drawn.
    Outline,
    /// The rectangle area is filled with some color.
    Fill,
}

/// Unique Kind for the widget.
pub const KIND: widget::Kind = "Rectangle";


impl Rectangle {

    /// Build a rectangle with the dimensions and style.
    pub fn styled(dim: Dimensions, style: Style) -> Self {
        Rectangle {
            common: widget::CommonBuilder::new(),
            style: style,
        }.wh(dim)
    }

    /// Build a new filled rectangle.
    pub fn fill(dim: Dimensions) -> Self {
        Rectangle::styled(dim, Style::fill())
    }

    /// Build a new filled rectangle widget filled with the given color.
    pub fn fill_with(dim: Dimensions, color: Color) -> Self {
        Rectangle::styled(dim, Style::fill_with(color))
    }

    /// Build a new outlined rectangle widget.
    pub fn outline(dim: Dimensions) -> Self {
        Rectangle::styled(dim, Style::outline())
    }

    /// Build an outlined rectangle rather than a filled one.
    pub fn outline_styled(dim: Dimensions, line_style: LineStyle) -> Self {
        Rectangle::styled(dim, Style::outline_styled(line_style))
    }

}


impl Widget for Rectangle {
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> &'static str {
        KIND
    }

    fn init_state(&self) -> State {
        State {
            kind: Kind::Fill,
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Rectangle.
    fn update<B: Backend>(self, args: widget::UpdateArgs<Self, B>) {
        let widget::UpdateArgs { state, style, .. } = args;

        let kind = match *style {
            Style::Fill(_) => Kind::Fill,
            Style::Outline(_) => Kind::Outline,
        };

        if state.view().kind != kind {
            state.update(|state| state.kind = kind);
        }
    }

}


impl Colorable for Rectangle {
    fn color(mut self, color: Color) -> Self {
        self.style.set_color(color);
        self
    }
}
