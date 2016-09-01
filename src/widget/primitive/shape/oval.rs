//! A simple, non-interactive widget for drawing a single **Oval**.

use {Color, Colorable, Dimensions, Sizeable, Widget};
use super::Style as Style;
use widget;


/// A simple, non-interactive widget for drawing a single **Oval**.
#[derive(Copy, Clone, Debug)]
pub struct Oval {
    /// Data necessary and common for all widget builder types.
    pub common: widget::CommonBuilder,
    /// Unique styling.
    pub style: Style,
}

/// Unique state for the **Oval**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct State;


impl Oval {

    /// Build an **Oval** with the given dimensions and style.
    pub fn styled(dim: Dimensions, style: Style) -> Self {
        Oval {
            common: widget::CommonBuilder::new(),
            style: style,
        }.wh(dim)
    }

    /// Build a new **Fill**ed **Oval**.
    pub fn fill(dim: Dimensions) -> Self {
        Oval::styled(dim, Style::fill())
    }

    /// Build a new **Oval** **Fill**ed with the given color.
    pub fn fill_with(dim: Dimensions, color: Color) -> Self {
        Oval::styled(dim, Style::fill_with(color))
    }

    /// Build a new **Outline**d **Oval** widget.
    pub fn outline(dim: Dimensions) -> Self {
        Oval::styled(dim, Style::outline())
    }

    /// Build a new **Oval** **Outline**d with the given style.
    pub fn outline_styled(dim: Dimensions, line_style: widget::line::Style) -> Self {
        Oval::styled(dim, Style::outline_styled(line_style))
    }

}


impl Widget for Oval {
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
        State
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn update(self, _args: widget::UpdateArgs<Self>) -> Self::Event {
        // Nothing to be updated here.
    }

}


impl Colorable for Oval {
    fn color(mut self, color: Color) -> Self {
        self.style.set_color(color);
        self
    }
}
