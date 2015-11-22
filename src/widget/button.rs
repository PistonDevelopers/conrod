
use {Label, NodeIndex, FramedRectangle, Ui};
use color::{Color, Colorable};
use frame::Frameable;
use graphics::character::CharacterCache;
use label::{FontSize, Labelable};
use mouse::Mouse;
use position::{Dimension, Positionable};
use theme::Theme;
use widget::{self, Widget};


/// A pressable button widget whose reaction is triggered upon release.
pub struct Button<'a, F> {
    common: widget::CommonBuilder,
    maybe_label: Option<&'a str>,
    maybe_react: Option<F>,
    /// Unique styling for the Button.
    pub style: Style,
    enabled: bool,
}

/// Styling for the Button, necessary for constructing its renderable Element.
#[allow(missing_copy_implementations)]
#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    /// Color of the Button's pressable area.
    pub maybe_color: Option<Color>,
    /// Width of the frame surrounding the button
    pub maybe_frame: Option<f64>,
    /// The color of the frame.
    pub maybe_frame_color: Option<Color>,
    /// The color of the Button's label.
    pub maybe_label_color: Option<Color>,
    /// The font size of the Button's label.
    pub maybe_label_font_size: Option<u32>,
}

/// Represents the state of the Button widget.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    maybe_rectangle_idx: Option<NodeIndex>,
    maybe_label_idx: Option<NodeIndex>,
    interaction: Interaction,
}

/// Unique kind for the widget.
pub const KIND: widget::Kind = "Button";

/// Represents an interaction with the Button widget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Interaction {
    Normal,
    Highlighted,
    Clicked,
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
fn get_new_interaction(is_over: bool, prev: Interaction, mouse: Mouse) -> Interaction {
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


impl<'a, F> Button<'a, F> {

    /// Create a button context to be built upon.
    pub fn new() -> Button<'a, F> {
        Button {
            common: widget::CommonBuilder::new(),
            maybe_react: None,
            maybe_label: None,
            style: Style::new(),
            enabled: true,
        }
    }

    /// Set the reaction for the Button. The reaction will be triggered upon release of the button.
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


impl<'a, F> Widget for Button<'a, F> where F: FnMut() {
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
            maybe_rectangle_idx: None,
            maybe_label_idx: None,
            interaction: Interaction::Normal,
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

    /// Update the state of the Button.
    fn update<C: CharacterCache>(self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { idx, state, style, rect, mut ui, .. } = args;
        let Button { enabled, maybe_label, mut maybe_react, .. } = self;
        let maybe_mouse = ui.input().maybe_mouse;

        // Check whether or not a new interaction has occurred.
        let new_interaction = match (enabled, maybe_mouse) {
            (false, _) | (true, None) => Interaction::Normal,
            (true, Some(mouse)) => {
                let is_over = rect.is_over(mouse.xy);
                get_new_interaction(is_over, state.view().interaction, mouse)
            },
        };

        // Capture the mouse if it was clicked, uncapture if it was released.
        match (state.view().interaction, new_interaction) {
            (Interaction::Highlighted, Interaction::Clicked) => { ui.capture_mouse(); },
            (Interaction::Clicked, Interaction::Highlighted) |
            (Interaction::Clicked, Interaction::Normal)      => { ui.uncapture_mouse(); },
            _ => (),
        }

        // If the mouse was released over button, react.
        if let (Interaction::Clicked, Interaction::Highlighted) =
            (state.view().interaction, new_interaction) {
            if let Some(ref mut react) = maybe_react { react() }
        }

        // FramedRectangle widget.
        let rectangle_idx = state.view().maybe_rectangle_idx
            .unwrap_or_else(|| ui.new_unique_node_index());
        let dim = rect.dim();
        let frame = style.frame(ui.theme());
        let color = new_interaction.color(style.color(ui.theme()));
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
            Label::new(label)
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


impl<'a, F> Colorable for Button<'a, F> {
    fn color(mut self, color: Color) -> Self {
        self.style.maybe_color = Some(color);
        self
    }
}

impl<'a, F> Frameable for Button<'a, F> {
    fn frame(mut self, width: f64) -> Self {
        self.style.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.style.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, F> Labelable<'a> for Button<'a, F> {
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

