//! The `Button` widget and related items.

use {
    Color,
    Colorable,
    FontSize,
    Borderable,
    Labelable,
    Positionable,
    Scalar,
    Widget,
};
use widget;


/// A pressable button widget whose reaction is triggered upon release.
#[derive(Clone)]
pub struct Button<'a> {
    common: widget::CommonBuilder,
    maybe_label: Option<&'a str>,
    /// Unique styling for the Button.
    pub style: Style,
    /// Whether or not user input is enabled.
    enabled: bool,
}

widget_style!{
    /// Unique styling for the Button.
    style Style {
        /// Color of the Button's pressable area.
        - color: Color { theme.shape_color }
        /// Width of the border surrounding the button
        - border: Scalar { theme.border_width }
        /// The color of the border.
        - border_color: Color { theme.border_color }
        /// The color of the Button's label.
        - label_color: Color { theme.label_color }
        /// The font size of the Button's label.
        - label_font_size: FontSize { theme.font_size_medium }
    }
}

/// Represents the state of the Button widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    rectangle_idx: widget::IndexSlot,
    label_idx: widget::IndexSlot,
}

/// The `Event` type yielded by the `Button` widget.
///
/// Represents the number of times that the `Button` has been clicked with the left mouse button
/// since the last update.
#[derive(Clone, Debug)]
#[allow(missing_copy_implementations)]
pub struct TimesClicked(pub u16);


impl TimesClicked {
    /// `true` if the `Button` was clicked one or more times.
    pub fn was_clicked(self) -> bool { self.0 > 0 }
}

impl Iterator for TimesClicked {
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 > 0 {
            self.0 -= 1;
            Some(())
        } else {
            None
        }
    }
}


impl<'a> Button<'a> {

    /// Create a button context to be built upon.
    pub fn new() -> Self {
        Button {
            common: widget::CommonBuilder::new(),
            maybe_label: None,
            style: Style::new(),
            enabled: true,
        }
    }

    builder_methods!{
        pub enabled { enabled = bool }
    }
}


impl<'a> Widget for Button<'a> {
    type State = State;
    type Style = Style;
    type Event = TimesClicked;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self) -> State {
        State {
            rectangle_idx: widget::IndexSlot::new(),
            label_idx: widget::IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Button.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { idx, state, style, rect, mut ui, .. } = args;

        let (color, times_clicked) = {
            let input = ui.widget_input(idx);
            let color = style.color(ui.theme());
            let color = input.mouse().map_or(color, |mouse| {
                if mouse.buttons.left().is_down() {
                    color.clicked()
                } else {
                    color.highlighted()
                }
            });
            let times_clicked = input.clicks().left().count() as u16;
            (color, times_clicked)
        };

        // BorderedRectangle widget.
        let rectangle_idx = state.rectangle_idx.get(&mut ui);
        let dim = rect.dim();
        let border = style.border(ui.theme());
        let border_color = style.border_color(ui.theme());
        widget::BorderedRectangle::new(dim)
            .middle_of(idx)
            .graphics_for(idx)
            .color(color)
            .border(border)
            .border_color(border_color)
            .set(rectangle_idx, &mut ui);

        // Label widget.
        if let Some(label) = self.maybe_label {
            let label_idx = state.label_idx.get(&mut ui);
            let color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            widget::Text::new(label)
                .middle_of(rectangle_idx)
                .graphics_for(idx)
                .color(color)
                .font_size(font_size)
                .set(label_idx, &mut ui);
        }

        TimesClicked(times_clicked)
    }

}


impl<'a> Colorable for Button<'a> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a> Borderable for Button<'a> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a> Labelable<'a> for Button<'a> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
