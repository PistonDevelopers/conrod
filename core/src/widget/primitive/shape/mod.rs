//! A module encompassing the primitive 2D shape widgets.

use color::Color;
use theme::Theme;
use widget;

pub mod circle;
pub mod oval;
pub mod polygon;
pub mod rectangle;
pub mod triangles;


/// The style for some 2D shape.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Style {
    /// The outline of the shape with this style.
    Outline(widget::line::Style),
    /// A rectangle filled with this color.
    Fill(Option<Color>),
}


impl Style {

    /// A default `Fill` style.
    pub fn fill() -> Self {
        Style::Fill(None)
    }

    /// A `Fill` style with some given `Color`.
    pub fn fill_with(color: Color) -> Self {
        Style::Fill(Some(color))
    }

    /// A default `Outline` style.
    pub fn outline() -> Self {
        Style::Outline(widget::line::Style::new())
    }

    /// A default `Outline` style.
    pub fn outline_styled(line_style: widget::line::Style) -> Self {
        Style::Outline(line_style)
    }

    /// The style with some given Color.
    pub fn color(mut self, color: Color) -> Self {
        self.set_color(color);
        self
    }

    /// Set the color for the style.
    pub fn set_color(&mut self, color: Color) {
        match *self {
            Style::Fill(ref mut maybe_color) => *maybe_color = Some(color),
            Style::Outline(ref mut line_style) => line_style.set_color(color),
        }
    }

    /// Get the color of the Rectangle.
    pub fn get_color(&self, theme: &Theme) -> Color {
        match *self {
            Style::Fill(maybe_color) => maybe_color.unwrap_or(theme.shape_color),
            Style::Outline(style) => style.get_color(theme),
        }
    }

}
