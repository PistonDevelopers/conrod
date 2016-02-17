
use {
    Backend,
    Color,
    Colorable,
    FontSize,
    Frameable,
    FramedRectangle,
    IndexSlot,
    Labelable,
    Mouse,
    Positionable,
    Scalar,
    Text,
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

/// Unique kind for the widget type.
pub const KIND: widget::Kind = "Toggle";

widget_style!{
    KIND;
    /// Styling for the Toggle including coloring, framing and labelling.
    style Style {
        /// Color of the Toggle's pressable area.
        - color: Color { theme.shape_color }
        /// The width of the rectangular frame surrounding the Toggle.
        - frame: Scalar { theme.frame_width }
        /// The color of the Toggle's frame.
        - frame_color: Color { theme.frame_color }
        /// The color of the Toggle's Text label.
        - label_color: Color { theme.label_color }
        /// The font size for the Toggle's Text label.
        - label_font_size: FontSize { theme.font_size_medium }
    }
}

/// The way in which the Toggle is being interacted with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Interaction {
    Normal,
    Highlighted,
    Clicked,
}

/// The state of the Toggle.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    value: bool,
    interaction: Interaction,
    rectangle_idx: IndexSlot,
    label_idx: IndexSlot,
}


impl Interaction {
    /// Alter the widget color depending on the state.
    fn color(&self, color: Color) -> Color {
        match *self {
            Interaction::Normal => color,
            Interaction::Highlighted => color.highlighted(),
            Interaction::Clicked => color.clicked(),
        }
    }
}


/// Check the current state of the button.
fn get_new_interaction(is_over: bool,
                       prev: Interaction,
                       mouse: Mouse) -> Interaction {
    use mouse::ButtonPosition::{Down, Up};
    use self::Interaction::{Normal, Highlighted, Clicked};
    match (is_over, prev, mouse.left.position) {
        (true,  Normal,  Down) => Normal,
        (true,  _,       Down) => Clicked,
        (true,  _,       Up)   => Highlighted,
        (false, Clicked, Down) => Clicked,
        _                      => Normal,
    }
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

impl<'a, B, F> Widget<B> for Toggle<'a, F>
    where B: Backend,
          F: FnOnce(bool),
{
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
            value: self.value,
            interaction: Interaction::Normal,
            rectangle_idx: IndexSlot::new(),
            label_idx: IndexSlot::new(),
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    /// Update the state of the Toggle.
    fn update(self, args: widget::UpdateArgs<Self, B>) {
        let widget::UpdateArgs { idx, state, style, rect, mut ui, .. } = args;
        let Toggle { value, enabled, maybe_label, maybe_react, .. } = self;
        let maybe_mouse = ui.input().maybe_mouse;

        // Check whether or not a new interaction has occurred.
        let new_interaction = match (enabled, maybe_mouse) {
            (false, _) | (true, None) => Interaction::Normal,
            (true, Some(mouse)) => {
                let is_over = rect.is_over(mouse.xy);
                get_new_interaction(is_over, state.view().interaction, mouse)
            },
        };

        // Capture the mouse if clicked, uncapture if released.
        match (state.view().interaction, new_interaction) {
            (Interaction::Highlighted, Interaction::Clicked) => { ui.capture_mouse(); },
            (Interaction::Clicked, Interaction::Highlighted) |
            (Interaction::Clicked, Interaction::Normal)      => { ui.uncapture_mouse(); },
            _ => (),
        }

        // React and determine the new value.
        let new_value = match (state.view().interaction, new_interaction) {
            (Interaction::Clicked, Interaction::Highlighted) => {
                let new_value = !value;
                if let Some(react) = maybe_react {
                    react(new_value)
                }
                new_value
            },
            _ => value,
        };

        // FramedRectangle widget.
        let rectangle_idx = state.view().rectangle_idx.get(&mut ui);
        let dim = rect.dim();
        let frame = style.frame(ui.theme());
        let color = {
            let color = style.color(ui.theme());
            let color = if new_value { color } else { color.with_luminance(0.1) };
            new_interaction.color(color)
        };
        let frame_color = style.frame_color(ui.theme());
        FramedRectangle::new(dim)
            .middle_of(idx)
            .graphics_for(idx)
            .color(color)
            .frame(frame)
            .frame_color(frame_color)
            .set(rectangle_idx, &mut ui);

        // Label widget.
        if let Some(label) = maybe_label {
            let label_idx = state.view().label_idx.get(&mut ui);
            let color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            Text::new(label)
                .middle_of(rectangle_idx)
                .graphics_for(idx)
                .color(color)
                .font_size(font_size)
                .set(label_idx, &mut ui);
        }

        // If there has been a change in interaction, set the new one.
        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        // If the value has changed, update our state.
        if state.view().value != new_value {
            state.update(|state| state.value = new_value);
        }
    }
}


impl<'a, F> Colorable for Toggle<'a, F> {
    builder_method!(color { style.color = Some(Color) });
}

impl<'a, F> Frameable for Toggle<'a, F> {
    builder_methods!{
        frame { style.frame = Some(Scalar) }
        frame_color { style.frame_color = Some(Color) }
    }
}

impl<'a, F> Labelable<'a> for Toggle<'a, F> {
    builder_methods!{
        label { maybe_label = Some(&'a str) }
        label_color { style.label_color = Some(Color) }
        label_font_size { style.label_font_size = Some(FontSize) }
    }
}
