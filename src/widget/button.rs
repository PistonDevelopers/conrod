
use Scalar;
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use graphics::character::CharacterCache;
use label::{FontSize, Labelable};
use mouse::Mouse;
use position::{self, Positionable};
use theme::Theme;
use ui::GlyphCache;
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
    maybe_label: Option<String>,
    interaction: Interaction,
}

/// Represents an interaction with the Button widget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Interaction {
    Normal,
    Highlighted,
    Clicked,
}


impl State {
    /// Alter the widget color depending on the state.
    fn color(&self, color: Color) -> Color {
        match self.interaction {
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

    fn common(&self) -> &widget::CommonBuilder { &self.common }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder { &mut self.common }
    fn unique_kind(&self) -> &'static str { "Button" }
    fn init_state(&self) -> State {
        State { maybe_label: None, interaction: Interaction::Normal }
    }
    fn style(&self) -> Style { self.style.clone() }

    fn capture_mouse(prev: &State, new: &State) -> bool {
        match (prev.interaction, new.interaction) {
            (Interaction::Highlighted, Interaction::Clicked) => true,
            _ => false,
        }
    }

    fn uncapture_mouse(prev: &State, new: &State) -> bool {
        match (prev.interaction, new.interaction) {
            (Interaction::Clicked, Interaction::Highlighted) => true,
            (Interaction::Clicked, Interaction::Normal) => true,
            _ => false,
        }
    }

    fn default_width<C: CharacterCache>(&self, theme: &Theme, _: &GlyphCache<C>) -> Scalar {
        const DEFAULT_WIDTH: Scalar = 64.0;
        self.common.maybe_width.or(theme.maybe_button.as_ref().map(|default| {
            default.common.maybe_width.unwrap_or(DEFAULT_WIDTH)
        })).unwrap_or(DEFAULT_WIDTH)
    }

    fn default_height(&self, theme: &Theme) -> Scalar {
        const DEFAULT_HEIGHT: Scalar = 64.0;
        self.common.maybe_height.or(theme.maybe_button.as_ref().map(|default| {
            default.common.maybe_height.unwrap_or(DEFAULT_HEIGHT)
        })).unwrap_or(DEFAULT_HEIGHT)
    }

    /// Update the state of the Button.
    fn update<C>(mut self, args: widget::UpdateArgs<Self, C>) -> Option<State>
        where C: CharacterCache
    {
        let widget::UpdateArgs { prev_state, rect, ui, .. } = args;
        let widget::State { ref state, .. } = *prev_state;
        let maybe_mouse = ui.input().maybe_mouse;

        // Check whether or not a new interaction has occurred.
        let new_interaction = match (self.enabled, maybe_mouse) {
            (false, _) | (true, None) => Interaction::Normal,
            (true, Some(mouse)) => {
                let is_over = rect.is_over(mouse.xy);
                get_new_interaction(is_over, state.interaction, mouse)
            },
        };

        // If the mouse was released over button, react.
        if let (Interaction::Clicked, Interaction::Highlighted) =
            (state.interaction, new_interaction) {
            if let Some(ref mut react) = self.maybe_react { react() }
        }

        // A function for constructing a new state.
        let new_state = || {
            State {
                maybe_label: self.maybe_label.as_ref().map(|label| label.to_string()),
                interaction: new_interaction,
            }
        };

        // Check whether or not the state has changed since the previous update.
        let state_has_changed = state.interaction != new_interaction
            || state.maybe_label.as_ref().map(|string| &string[..]) != self.maybe_label;

        // Construct the new state if there was a change.
        if state_has_changed { Some(new_state()) } else { None }
    }

    /// Construct an Element from the given Button State.
    fn draw<C>(args: widget::DrawArgs<Self, C>) -> Element
        where C: CharacterCache
    {
        use elmesque::form::{self, collage, text};

        let widget::DrawArgs { state, style, theme, rect, .. } = args;
        let xy = rect.xy();
        let dim = rect.dim();

        // Retrieve the styling for the Element..
        let color = state.color(style.color(theme));
        let frame = style.frame(theme);
        let frame_color = style.frame_color(theme);

        // Construct the frame and inner rectangle forms.
        let frame_form = form::rect(dim[0], dim[1]).filled(frame_color);
        let (inner_w, inner_h) = (dim[0] - frame * 2.0, dim[1] - frame * 2.0);
        let pressable_form = form::rect(inner_w, inner_h).filled(color);

        // Construct the label's Form.
        let maybe_label_form = state.maybe_label.as_ref().map(|label_text| {
            use elmesque::text::Text;
            let label_color = style.label_color(theme);
            let size = style.label_font_size(theme);
            text(Text::from_string(label_text.to_string()).color(label_color).height(size as f64))
                .shift(xy[0].floor(), xy[1].floor())
        });

        // Construct the button's Form.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(pressable_form))
            .map(|form| form.shift(xy[0], xy[1]))
            .chain(maybe_label_form);

        // Turn the form into a renderable Element.
        collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
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
        self.maybe_color.or(theme.maybe_button.as_ref().map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_button.as_ref().map(|default| {
            default.style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_button.as_ref().map(|default| {
            default.style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.maybe_button.as_ref().map(|default| {
            default.style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.maybe_button.as_ref().map(|default| {
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

