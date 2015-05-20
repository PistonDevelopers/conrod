
use canvas::CanvasId;
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use graphics::character::CharacterCache;
use graphics::math::Scalar;
use label::{FontSize, Labelable};
use mouse::Mouse;
use position::{self, Depth, Dimensions, HorizontalAlign, Point, Position, VerticalAlign};
use theme::Theme;
use ui::{GlyphCache, UserInput};
use widget::{self, Widget};


/// A pressable widget for toggling the state of a bool. Like the button widget, it's reaction is
/// triggered upon release and will return the new bool state. Note that the toggle will not
/// mutate the bool for you, you should do this yourself within the react closure.
pub struct Toggle<'a, F> {
    pos: Position,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    depth: Depth,
    value: bool,
    maybe_react: Option<F>,
    maybe_label: Option<&'a str>,
    style: Style,
    enabled: bool,
    maybe_canvas_id: Option<CanvasId>,
}

/// Styling for the Toggle, necessary for constructing its renderable Element.
#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    pub maybe_width: Option<Scalar>,
    pub maybe_height: Option<Scalar>,
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
    use mouse::ButtonState::{Down, Up};
    use self::Interaction::{Normal, Highlighted, Clicked};
    match (is_over, prev, mouse.left) {
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
            pos: Position::default(),
            maybe_h_align: None,
            maybe_v_align: None,
            depth: 0.0,
            maybe_react: None,
            maybe_label: None,
            value: value,
            style: Style::new(),
            enabled: true,
            maybe_canvas_id: None,
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

    /// Set which Canvas to attach the Widget to. Note that you can also attach a widget to a
    /// Canvas by using the canvas placement `Positionable` methods.
    pub fn canvas(mut self, id: CanvasId) -> Self {
        self.maybe_canvas_id = Some(id);
        self
    }

}

impl<'a, F> Widget for Toggle<'a, F>
    where
        F: FnMut(bool),
{
    type State = State;
    type Style = Style;
    fn unique_kind(&self) -> &'static str { "Toggle" }
    fn init_state(&self) -> State {
        State {
            value: self.value,
            interaction: Interaction::Normal,
            maybe_label: None,
        }
    }
    fn style(&self) -> Style { self.style.clone() }
    fn canvas_id(&self) -> Option<CanvasId> { self.maybe_canvas_id }

    /// Update the state of the Toggle.
    fn update<'b, C>(mut self,
                     prev_state: &widget::State<State>,
                     xy: Point,
                     dim: Dimensions,
                     input: UserInput<'b>,
                     _style: &Style,
                     _theme: &Theme,
                     _glyph_cache: &GlyphCache<C>) -> Option<State>
        where
            C: CharacterCache,
    {
        use utils::is_over_rect;

        let widget::State { ref state, .. } = *prev_state;
        let maybe_mouse = input.maybe_mouse.map(|mouse| mouse.relative_to(xy));

        // Check whether or not a new interaction has occurred.
        let new_interaction = match (self.enabled, maybe_mouse) {
            (false, _) | (true, None) => Interaction::Normal,
            (true, Some(mouse)) => {
                let is_over = is_over_rect([0.0, 0.0], mouse.xy, dim);
                get_new_interaction(is_over, state.interaction, mouse)
            },
        };

        // React and determine the new value.
        let new_value = match (state.interaction, new_interaction) {
            (Interaction::Clicked, Interaction::Highlighted) => {
                let new_value = !self.value;
                if let Some(ref mut react) = self.maybe_react { react(!self.value) }
                new_value
            },
            _ => self.value,
        };

        // A function for constructing a new Toggle State.
        let new_state = || {
            State {
                maybe_label: self.maybe_label.as_ref().map(|label| label.to_string()),
                value: new_value,
                interaction: new_interaction,
            }
        };

        // Check whether or not the state has changed since the previous update.
        let state_has_changed = state.interaction != new_interaction
            || state.value != self.value
            || state.maybe_label.as_ref().map(|string| &string[..]) != self.maybe_label;

        // Construct the new state if there was a change.
        if state_has_changed { Some(new_state()) } else { None }
    }

    /// Construct an Element from the given Toggle State.
    fn draw<C>(new_state: &widget::State<State>,
               style: &Style,
               theme: &Theme,
               _glyph_cache: &GlyphCache<C>) -> Element
        where
            C: CharacterCache,
    {
        use elmesque::form::{collage, rect, text};

        let widget::State { ref state, dim, xy, .. } = *new_state;

        // Construct the frame and pressable forms.
        let frame = style.frame(theme);
        let frame_color = style.frame_color(theme);
        let (inner_w, inner_h) = (dim[0] - frame * 2.0, dim[1] - frame * 2.0);
        let frame_form = rect(dim[0], dim[1]).filled(frame_color);
        let color = style.color(theme);
        let color = state.color(if state.value { color }
                                    else { color.with_luminance(0.1) });
        let pressable_form = rect(inner_w, inner_h).filled(color);

        // Construct the label's Form.
        let maybe_label_form = state.maybe_label.as_ref().map(|label_text| {
            use elmesque::text::Text;
            let label_color = style.label_color(theme);
            let font_size = style.label_font_size(theme) as f64;
            text(Text::from_string(label_text.clone()).color(label_color).height(font_size))
                .shift(xy[0].floor(), xy[1].floor())
        });

        // Chain the Forms and shift them into position.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(pressable_form).into_iter())
            .map(|form| form.shift(xy[0], xy[1]))
            .chain(maybe_label_form.into_iter());

        // Collect the Forms into a renderable Element.
        collage(dim[0] as i32, dim[1] as i32, form_chain.collect())
    }

}


impl Style {

    /// Construct the default Style.
    pub fn new() -> Style {
        Style {
            maybe_width: None,
            maybe_height: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
        }
    }

    /// Get the width of the Widget.
    pub fn width(&self, theme: &Theme) -> Scalar {
        const DEFAULT_WIDTH: Scalar = 64.0;
        self.maybe_width.or(theme.maybe_toggle.as_ref().map(|style| {
            style.maybe_width.unwrap_or(DEFAULT_WIDTH)
        })).unwrap_or(DEFAULT_WIDTH)
    }

    /// Get the height of the Widget.
    pub fn height(&self, theme: &Theme) -> Scalar {
        const DEFAULT_HEIGHT: Scalar = 64.0;
        self.maybe_height.or(theme.maybe_toggle.as_ref().map(|style| {
            style.maybe_height.unwrap_or(DEFAULT_HEIGHT)
        })).unwrap_or(DEFAULT_HEIGHT)
    }

    /// Get the Color for an Element.
    pub fn color(&self, theme: &Theme) -> Color {
        self.maybe_color.or(theme.maybe_toggle.as_ref().map(|style| {
            style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_toggle.as_ref().map(|style| {
            style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_toggle.as_ref().map(|style| {
            style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.maybe_toggle.as_ref().map(|style| {
            style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.maybe_toggle.as_ref().map(|style| {
            style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
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

impl<'a, F> position::Positionable for Toggle<'a, F> {
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
    fn get_position(&self) -> Position { self.pos }
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        Toggle { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        Toggle { maybe_v_align: Some(v_align), ..self }
    }
    fn get_horizontal_align(&self, theme: &Theme) -> HorizontalAlign {
        self.maybe_h_align.unwrap_or(theme.align.horizontal)
    }
    fn get_vertical_align(&self, theme: &Theme) -> VerticalAlign {
        self.maybe_v_align.unwrap_or(theme.align.vertical)
    }
    fn depth(mut self, depth: Depth) -> Self {
        self.depth = depth;
        self
    }
    fn get_depth(&self) -> Depth { self.depth }
}

impl<'a, F> position::Sizeable for Toggle<'a, F> {
    fn width(mut self, w: Scalar) -> Self {
        self.style.maybe_width = Some(w);
        self
    }
    fn height(mut self, h: Scalar) -> Self {
        self.style.maybe_height = Some(h);
        self
    }
    fn get_width<C: CharacterCache>(&self, theme: &Theme, _: &GlyphCache<C>) -> Scalar {
        self.style.width(theme)
    }
    fn get_height(&self, theme: &Theme) -> Scalar {
        self.style.height(theme)
    }
}

