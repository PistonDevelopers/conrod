//! A button that allows for toggling boolean state.

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


/// A pressable widget for toggling the state of a bool.
///
/// Like the Button widget, it's reaction is triggered upon release and will return the new bool
/// state.
///
/// Note that the Toggle will not mutate the bool for you, you should do this yourself within the
/// react function.
pub struct Toggle<'a, F> {
    common: widget::CommonBuilder,
    value: bool,
    /// Set the reaction for the Toggle. It will be triggered upon release of the button.
    pub maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
    /// If true, will allow user inputs. If false, will disallow user inputs.
    pub enabled: bool,
}

widget_style!{
    /// Styling for the Toggle including coloring, bordering and labelling.
    style Style {
        /// Color of the Toggle's pressable area.
        - color: Color { theme.shape_color }
        /// The width of the rectangular border surrounding the Toggle.
        - border: Scalar { theme.border_width }
        /// The color of the Toggle's border.
        - border_color: Color { theme.border_color }
        /// The color of the Toggle's Text label.
        - label_color: Color { theme.label_color }
        /// The font size for the Toggle's Text label.
        - label_font_size: FontSize { theme.font_size_medium }
    }
}

/// The state of the Toggle.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    value: bool,
    rectangle_idx: widget::IndexSlot,
    label_idx: widget::IndexSlot,
}


impl<'a, F> Toggle<'a, F> {

    /// Construct a new Toggle widget.
    pub fn new(value: bool) -> Toggle<'a, F> {
        Toggle {
            common: widget::CommonBuilder::new(),
            maybe_react: None,
            maybe_label: None,
            value: value,
            style: Style::new(),
            enabled: true,
        }
    }

    builder_methods!{
        pub react { maybe_react = Some(F) }
        pub enabled { enabled = bool }
    }

}

impl<'a, F> Widget for Toggle<'a, F>
    where F: FnOnce(bool),
{
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn init_state(&self) -> State {
        State {
            value: self.value,
            rectangle_idx: widget::IndexSlot::new(),
            label_idx: widget::IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Toggle.
    fn update(self, args: widget::UpdateArgs<Self>) {
        let widget::UpdateArgs { idx, state, style, rect, mut ui, .. } = args;
        let Toggle { value, enabled, maybe_label, maybe_react, .. } = self;

        let new_value = if ui.widget_input(idx).clicks().left().next().is_some() && enabled {
            let new_value = !value;
            if let Some(react) = maybe_react {
                react(new_value)
            }
            new_value
        } else {
            value
        };

        // If the value has changed, update our state.
        if state.value != new_value {
            state.update(|state| state.value = new_value);
        }

        // BorderedRectangle widget.
        let rectangle_idx = state.rectangle_idx.get(&mut ui);
        let dim = rect.dim();
        let border = style.border(ui.theme());
        let color = {
            let color = style.color(ui.theme());
            let color = if new_value { color } else { color.with_luminance(0.1) };
            match ui.widget_input(idx).mouse() {
                Some(mouse) =>
                    if mouse.buttons.left().is_down() { color.clicked() }
                    else { color.highlighted() },
                None => color,
            }
        };
        let border_color = style.border_color(ui.theme());
        widget::BorderedRectangle::new(dim)
            .middle_of(idx)
            .graphics_for(idx)
            .color(color)
            .border(border)
            .border_color(border_color)
            .set(rectangle_idx, &mut ui);

        // Label widget.
        if let Some(label) = maybe_label {
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
    }
}


impl<'a, F> Colorable for Toggle<'a, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, F> Borderable for Toggle<'a, F> {
    builder_methods!{
        border { style.border = Some(Scalar) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a, F> Labelable<'a> for Toggle<'a, F> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
