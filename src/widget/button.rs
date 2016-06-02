use {
    Backend,
    Color,
    Colorable,
    FontSize,
    Frameable,
    FramedRectangle,
    IndexSlot,
    Labelable,
    Positionable,
    Scalar,
    Text,
    Widget,
};
use widget;


/// A pressable button widget whose reaction is triggered upon release.
pub struct Button<'a, F> {
    common: widget::CommonBuilder,
    maybe_label: Option<&'a str>,
    /// The reaction for the Button. The reaction will be triggered upon release of the button.
    maybe_react: Option<F>,
    /// Unique styling for the Button.
    pub style: Style,
    /// Whether or not user input is enabled.
    enabled: bool,
}

/// Unique kind for the widget.
pub const KIND: widget::Kind = "Button";

widget_style!{
    KIND;
    /// Unique styling for the Button.
    style Style {
        /// Color of the Button's pressable area.
        - color: Color { theme.shape_color }
        /// Width of the frame surrounding the button
        - frame: Scalar { theme.frame_width }
        /// The color of the frame.
        - frame_color: Color { theme.frame_color }
        /// The color of the Button's label.
        - label_color: Color { theme.label_color }
        /// The font size of the Button's label.
        - label_font_size: FontSize { theme.font_size_medium }
    }
}

/// Represents the state of the Button widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    rectangle_idx: IndexSlot,
    label_idx: IndexSlot,
}

impl<'a, F> Button<'a, F> {

    /// Create a button context to be built upon.
    pub fn new() -> Self {
        Button {
            common: widget::CommonBuilder::new(),
            maybe_react: None,
            maybe_label: None,
            style: Style::new(),
            enabled: true,
        }
    }

    builder_methods!{
        pub react { maybe_react = Some(F) }
        pub enabled { enabled = bool }
    }
}


impl<'a, F> Widget for Button<'a, F>
    where F: FnOnce(),
{
    type State = State;
    type Style = Style;

    fn common(&self) -> &widget::CommonBuilder {
        &self.common
    }

    fn common_mut(&mut self) -> &mut widget::CommonBuilder {
        &mut self.common
    }

    fn unique_kind(&self) -> widget::Kind {
        KIND
    }

    fn init_state(&self) -> State {
        State {
            rectangle_idx: IndexSlot::new(),
            label_idx: IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Button.
    fn update<B: Backend>(self, args: widget::UpdateArgs<Self, B>) {
        let widget::UpdateArgs { idx, state, style, rect, mut ui, .. } = args;

        let color = {
            let input = ui.widget_input(idx);
            if input.clicks().left().next().is_some() {
                if let Some(react) = self.maybe_react {
                    react()
                }
            }

            let color = style.color(ui.theme());
            input.mouse().map_or(color, |mouse| {
                if mouse.buttons.left().is_down() {
                    color.clicked()
                } else {
                    color.highlighted()
                }
            })
        };

        // FramedRectangle widget.
        let rectangle_idx = state.rectangle_idx.get(&mut ui);
        let dim = rect.dim();
        let frame = style.frame(ui.theme());
        let frame_color = style.frame_color(ui.theme());
        FramedRectangle::new(dim)
            .middle_of(idx)
            .graphics_for(idx)
            .color(color)
            .frame(frame)
            .frame_color(frame_color)
            .set(rectangle_idx, &mut ui);

        // Label widget.
        if let Some(label) = self.maybe_label {
            let label_idx = state.label_idx.get(&mut ui);
            let color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            Text::new(label)
                .middle_of(rectangle_idx)
                .graphics_for(idx)
                .color(color)
                .font_size(font_size)
                .set(label_idx, &mut ui);
        }

    }

}


impl<'a, F> Colorable for Button<'a, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, F> Frameable for Button<'a, F> {
    builder_methods!{
        frame { style.frame = Some(Scalar) }
        frame_color { style.frame_color = Some(Color) }
    }
}

impl<'a, F> Labelable<'a> for Button<'a, F> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
