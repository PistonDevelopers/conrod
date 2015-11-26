use {CharacterCache, LineStyle};
use color::{Color, Colorable};
use elmesque::Element;
use position::{Dimensions, Sizeable};
use super::Style as Style;
use widget::{self, Widget};
use widget::primitive::point_path;


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
        }.dim(dim)
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
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { state, style, .. } = args;

        let kind = match *style {
            Style::Fill(_) => Kind::Fill,
            Style::Outline(_) => Kind::Outline,
        };

        if state.view().kind != kind {
            state.update(|state| state.kind = kind);
        }
    }

    /// Construct an Element for the Rectangle.
    fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
        use elmesque::form::{self, collage};
        let widget::DrawArgs { rect, style, theme, .. } = args;
        match *style {

            // Draw a filled rectangle.
            Style::Fill(_) => {
                let (x, y, w, h) = rect.x_y_w_h();
                let color = style.get_color(theme);
                let form = form::rect(w, h).filled(color).shift(x, y);
                collage(w as i32, h as i32, vec![form])
            },

            // Draw only the outline of the rectangle.
            Style::Outline(line_style) => {
                let points = {
                    use std::iter::once;
                    let tl = [rect.x.start, rect.y.end];
                    let tr = [rect.x.end, rect.y.end];
                    let br = [rect.x.end, rect.y.start];
                    let bl = [rect.x.start, rect.y.start];
                    once(tl).chain(once(tr)).chain(once(br)).chain(once(bl)).chain(once(tl))
                };
                point_path::draw_lines(points, rect, line_style, theme)
            },
        }
    }

}


impl Colorable for Rectangle {
    fn color(mut self, color: Color) -> Self {
        self.style.set_color(color);
        self
    }
}

