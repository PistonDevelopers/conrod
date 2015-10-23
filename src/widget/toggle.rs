
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use graphics::character::CharacterCache;
use graphics::math::Scalar;
use label::{FontSize, Labelable};
use mouse::Mouse;
use theme::Theme;
use ui::GlyphCache;
use widget::{self, Widget};


/// A pressable widget for toggling the state of a bool. Like the button widget, it's reaction is
/// triggered upon release and will return the new bool state. Note that the toggle will not
/// mutate the bool for you, you should do this yourself within the react closure.
pub struct Toggle<'a, F> {
    common: widget::CommonBuilder,
    value: bool,
    maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
    enabled: bool,
}

/// Styling for the Toggle, necessary for constructing its renderable Element.
#[allow(missing_docs, missing_copy_implementations)]
#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    pub maybe_color: Option<Color>,
    pub maybe_frame: Option<Scalar>,
    pub maybe_frame_color: Option<Color>,
    pub maybe_label_color: Option<Color>,
    pub maybe_label_font_size: Option<u32>,
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
    maybe_label: Option<String>,
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

impl<'a, F> Widget for Toggle<'a, F> where F: FnMut(bool), {
    type State = State;
    type Style = Style;
    fn common(&self) -> &widget::CommonBuilder { &self.common }
    fn common_mut(&mut self) -> &mut widget::CommonBuilder { &mut self.common }
    fn unique_kind(&self) -> &'static str { "Toggle" }
    fn init_state(&self) -> State {
        State {
            value: self.value,
            interaction: Interaction::Normal,
            maybe_label: None,
        }
    }
    fn style(&self) -> Style { self.style.clone() }

    fn default_width<C: CharacterCache>(&self, theme: &Theme, _: &GlyphCache<C>) -> Scalar {
        const DEFAULT_WIDTH: Scalar = 64.0;
        theme.maybe_toggle.as_ref().map(|default| {
            default.common.maybe_width.unwrap_or(DEFAULT_WIDTH)
        }).unwrap_or(DEFAULT_WIDTH)
    }

    fn default_height(&self, theme: &Theme) -> Scalar {
        const DEFAULT_HEIGHT: Scalar = 64.0;
        theme.maybe_toggle.as_ref().map(|default| {
            default.common.maybe_height.unwrap_or(DEFAULT_HEIGHT)
        }).unwrap_or(DEFAULT_HEIGHT)
    }

    /// Update the state of the Toggle.
    fn update<C: CharacterCache>(mut self, args: widget::UpdateArgs<Self, C>) {
        let widget::UpdateArgs { state, rect, mut ui, .. } = args;

        let maybe_mouse = ui.input().maybe_mouse;

        // Check whether or not a new interaction has occurred.
        let new_interaction = match (self.enabled, maybe_mouse) {
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
                let new_value = !self.value;
                if let Some(ref mut react) = self.maybe_react { react(!self.value) }
                new_value
            },
            _ => self.value,
        };

        if state.view().interaction != new_interaction {
            state.update(|state| state.interaction = new_interaction);
        }

        if state.view().value != new_value {
            state.update(|state| state.value = new_value);
        }

        if state.view().maybe_label.as_ref().map(|label| &label[..]) != self.maybe_label {
            state.update(|state| {
                state.maybe_label = self.maybe_label.as_ref().map(|label| label.to_string());
            })
        }
    }

    /// Construct an Element from the given Toggle State.
    fn draw<C: CharacterCache>(args: widget::DrawArgs<Self, C>) -> Element {
        use elmesque::form::{self, collage, text};

        let widget::DrawArgs { rect, state, style, theme, .. } = args;

        // Construct the frame and pressable forms.
        let (x, y, w, h) = rect.x_y_w_h();
        let frame = style.frame(theme);
        let frame_color = style.frame_color(theme);
        let (inner_w, inner_h) = (w - frame * 2.0, h - frame * 2.0);
        let frame_form = form::rect(w, h).filled(frame_color);
        let color = style.color(theme);
        let color = state.color(if state.value { color }
                                    else { color.with_luminance(0.1) });
        let pressable_form = form::rect(inner_w, inner_h).filled(color);

        // Construct the label's Form.
        let maybe_label_form = state.maybe_label.as_ref().map(|label_text| {
            use elmesque::text::Text;
            let label_color = style.label_color(theme);
            let font_size = style.label_font_size(theme) as f64;
            text(Text::from_string(label_text.clone()).color(label_color).height(font_size))
                .shift(x.floor(), y.floor())
        });

        // Chain the Forms and shift them into position.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(pressable_form).into_iter())
            .map(|form| form.shift(x, y))
            .chain(maybe_label_form.into_iter());

        // Collect the Forms into a renderable Element.
        collage(w as i32, h as i32, form_chain.collect())
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
        self.maybe_color.or(theme.maybe_toggle.as_ref().map(|default| {
            default.style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_toggle.as_ref().map(|default| {
            default.style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_toggle.as_ref().map(|default| {
            default.style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.maybe_toggle.as_ref().map(|default| {
            default.style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.maybe_toggle.as_ref().map(|default| {
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

