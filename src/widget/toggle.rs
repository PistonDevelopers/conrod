
use {
    CharacterCache,
    Color,
    Colorable,
    Dimension,
    FontSize,
    Frameable,
    FramedRectangle,
    Labelable,
    Mouse,
    NodeIndex,
    Positionable,
    Scalar,
    Text,
    Theme,
    Ui,
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
    maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
    enabled: bool,
}

/// Styling for the Toggle including coloring, framing and labelling.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    /// Color of the Toggle's pressable area.
    pub maybe_color: Option<Color>,
    /// The width of the rectangular frame surrounding the Toggle.
    pub maybe_frame: Option<Scalar>,
    /// The color of the Toggle's frame.
    pub maybe_frame_color: Option<Color>,
    /// The color of the Toggle's Text label.
    pub maybe_label_color: Option<Color>,
    /// The font size for the Toggle's Text label.
    pub maybe_label_font_size: Option<u32>,
}

/// Unique kind for the widget type.
pub const KIND: widget::Kind = "Toggle";

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
    maybe_rectangle_idx: Option<NodeIndex>,
    maybe_label_idx: Option<NodeIndex>,
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

    /// Set the reaction for the Toggle. It will be triggered upon release of the button.
    pub fn react(mut self, reaction: F) -> Self {
        self.maybe_react = Some(reaction);
        self
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
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

    fn unique_kind(&self) -> &'static str {
        KIND
    }

    fn init_state(&self) -> State {
        State {
            value: self.value,
            interaction: Interaction::Normal,
            maybe_rectangle_idx: None,
            maybe_label_idx: None,
        }
    }

    fn style(&self) -> Style {
        self.style.clone()
    }

    fn default_x_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        widget::default_x_dimension(self, ui).unwrap_or(Dimension::Absolute(64.0))
    }

    fn default_y_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension {
        widget::default_y_dimension(self, ui).unwrap_or(Dimension::Absolute(64.0))
    }

    /// Update the state of the Toggle.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
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
        let rectangle_idx = state.view().maybe_rectangle_idx
            .unwrap_or_else(|| ui.new_unique_node_index());
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
        let maybe_label_idx = maybe_label.map(|label| {
            let label_idx = state.view().maybe_label_idx
                .unwrap_or_else(|| ui.new_unique_node_index());
            let color = style.label_color(ui.theme());
            let font_size = style.label_font_size(ui.theme());
            Text::new(label)
                .middle_of(rectangle_idx)
                .graphics_for(idx)
                .color(color)
                .font_size(font_size)
                .set(label_idx, &mut ui);
            label_idx
        });

        // If there has been a change in interaction, set the new one.
        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        // If the value has changed, update our state.
        if state.view().value != new_value {
            state.update(|state| state.value = new_value);
        }

        // If the rectangle index has changed, update it.
        if state.view().maybe_rectangle_idx != Some(rectangle_idx) {
            state.update(|state| state.maybe_rectangle_idx = Some(rectangle_idx));
        }

        // If the label index has changed, update it.
        if state.view().maybe_label_idx != maybe_label_idx {
            state.update(|state| state.maybe_label_idx = maybe_label_idx);
        }
    }
}


impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
        }
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.widget_style::<Self>(KIND).map(|default| {
            default.style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
        })).unwrap_or(theme.font_size_medium)
    }

}


impl<'a, F> Colorable for Toggle<'a, F> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a, F> Frameable for Toggle<'a, F> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, F> Labelable<'a> for Toggle<'a, F> {
    fn label(mut self, text: &'a str) -> Self {
        self.maybe_label = Some(text);
        self
    }

    fn label_color(mut self, color: Color) -> Self {
        self.style.maybe_label_color = Some(color);
        self
    }

    fn label_font_size(mut self, size: FontSize) -> Self {
        self.style.maybe_label_font_size = Some(size);
        self
    }
}

