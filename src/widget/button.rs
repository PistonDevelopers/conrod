
use color::{Color, Colorable};
use elmesque::Element;
use frame::Frameable;
use graphics::character::CharacterCache;
use label::{FontSize, Labelable};
use mouse::Mouse;
use position::{Depth, Dimensions, HorizontalAlign, Position, Positionable, VerticalAlign};
use theme::Theme;
use ui::{UiId, Ui};
use widget::{self, Widget};


/// A pressable button widget whose reaction is triggered upon release.
pub struct Button<'a, F> {
    pos: Position,
    dim: Dimensions,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    depth: Depth,
    maybe_label: Option<&'a str>,
    maybe_react: Option<F>,
    style: Style,
}

/// Styling for the Button, necessary for constructing its renderable Element.
#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Style {
    pub maybe_color: Option<Color>,
    pub maybe_frame: Option<f64>,
    pub maybe_frame_color: Option<Color>,
    pub maybe_label_color: Option<Color>,
    pub maybe_label_font_size: Option<u32>,
}

/// Represents the state of the Button widget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    Normal,
    Highlighted,
    Clicked,
}


impl State {
    /// Alter the widget color depending on the state.
    fn color(&self, color: Color) -> Color {
        match *self {
            State::Normal => color,
            State::Highlighted => color.highlighted(),
            State::Clicked => color.clicked(),
        }
    }
}


/// Check the current state of the button.
fn get_new_state(is_over: bool, prev: State, mouse: Mouse) -> State {
    use mouse::ButtonState::{Down, Up};
    use self::State::{Normal, Highlighted, Clicked};
    match (is_over, prev, mouse.left) {
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
            dim: [64.0, 64.0],
            pos: Position::default(),
            maybe_h_align: None,
            maybe_v_align: None,
            depth: 0.0,
            maybe_react: None,
            maybe_label: None,
            style: Style::new(),
        }
    }

    /// Set the reaction for the Button. The reaction will be triggered upon release of the button.
    pub fn react(mut self, reaction: F) -> Button<'a, F> {
        self.maybe_react = Some(reaction);
        self
    }

}


impl<'a, F> Widget for Button<'a, F>
    where
        F: FnMut()
{
    type State = State;
    type Style = Style;
    fn unique_kind(&self) -> &'static str { "Button" }
    fn init_state(&self) -> State { State::Normal }
    fn style(&self) -> Style { self.style.clone() }

    /// Update the state of the Button.
    fn update<C>(&mut self,
                 prev_state: &widget::State<State>,
                 _style: &Style,
                 ui_id: UiId,
                 ui: &mut Ui<C>) -> widget::State<State>
        where
            C: CharacterCache,
    {
        use utils::is_over_rect;
        let widget::State { state, .. } = *prev_state;
        let dim = self.dim;
        let h_align = self.maybe_h_align.unwrap_or(ui.theme.align.horizontal);
        let v_align = self.maybe_v_align.unwrap_or(ui.theme.align.vertical);
        let xy = ui.get_xy(self.pos, dim, h_align, v_align);
        let mouse = ui.get_mouse_state(ui_id).relative_to(xy);
        let is_over = is_over_rect([0.0, 0.0], mouse.xy, dim);
        let new_state = get_new_state(is_over, state, mouse);
        // React.
        if let (true, State::Clicked, State::Highlighted) = (is_over, state, new_state) {
            if let Some(ref mut react) = self.maybe_react { react() }
        }
        widget::State { state: new_state, xy: xy, depth: self.depth }
    }

    /// Construct an Element from the given Button State.
    fn draw<C>(&mut self,
               new_state: &widget::State<State>,
               style: &Style,
               _ui_id: UiId,
               ui: &mut Ui<C>) -> Element
        where
            C: CharacterCache,
    {
        use elmesque::form::{collage, rect, text};
        let widget::State { ref state, xy, .. } = *new_state;
        let dim = self.dim;
        let theme = &ui.theme;
        let maybe_label = self.maybe_label.take();

        // Retrieve the styling for the Element..
        let color = state.color(style.color(theme));
        let frame = style.frame(theme);
        let frame_color = style.frame_color(theme);

        // Construct the frame and inner rectangle forms.
        let frame_form = rect(dim[0], dim[1]).filled(frame_color);
        let (inner_w, inner_h) = (dim[0] - frame * 2.0, dim[1] - frame * 2.0);
        let pressable_form = rect(inner_w, inner_h).filled(color);

        // Construct the label's Form.
        let maybe_label_form = maybe_label.map(|label_text| {
            use elmesque::text::Text;
            let label_color = style.label_color(theme);
            let size = style.label_font_size(theme);
            text(Text::from_string(label_text.to_string()).color(label_color).height(size as f64))
                .shift(xy[0].floor(), xy[1].floor())
        });

        // Construct the button's Form.
        let form_chain = Some(frame_form).into_iter()
            .chain(Some(pressable_form).into_iter())
            .map(|form| form.shift(xy[0], xy[1]))
            .chain(maybe_label_form.into_iter());

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
        self.maybe_color.or(theme.maybe_button.as_ref().map(|style| {
            style.maybe_color.unwrap_or(theme.shape_color)
        })).unwrap_or(theme.shape_color)
    }

    /// Get the frame for an Element.
    pub fn frame(&self, theme: &Theme) -> f64 {
        self.maybe_frame.or(theme.maybe_button.as_ref().map(|style| {
            style.maybe_frame.unwrap_or(theme.frame_width)
        })).unwrap_or(theme.frame_width)
    }

    /// Get the frame Color for an Element.
    pub fn frame_color(&self, theme: &Theme) -> Color {
        self.maybe_frame_color.or(theme.maybe_button.as_ref().map(|style| {
            style.maybe_frame_color.unwrap_or(theme.frame_color)
        })).unwrap_or(theme.frame_color)
    }

    /// Get the label Color for an Element.
    pub fn label_color(&self, theme: &Theme) -> Color {
        self.maybe_label_color.or(theme.maybe_button.as_ref().map(|style| {
            style.maybe_label_color.unwrap_or(theme.label_color)
        })).unwrap_or(theme.label_color)
    }

    /// Get the label font size for an Element.
    pub fn label_font_size(&self, theme: &Theme) -> FontSize {
        self.maybe_label_font_size.or(theme.maybe_button.as_ref().map(|style| {
            style.maybe_label_font_size.unwrap_or(theme.font_size_medium)
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

impl<'a, F> Positionable for Button<'a, F> {
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        Button { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        Button { maybe_v_align: Some(v_align), ..self }
    }
}

impl<'a, F> ::position::Sizeable for Button<'a, F> {
    #[inline]
    fn width(self, w: f64) -> Self {
        let h = self.dim[1];
        Button { dim: [w, h], ..self }
    }
    #[inline]
    fn height(self, h: f64) -> Self {
        let w = self.dim[0];
        Button { dim: [w, h], ..self }
    }
}

