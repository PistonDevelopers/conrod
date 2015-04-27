
use color::{Color, Colorable};
use frame::Frameable;
use label::{FontSize, Labelable};
use mouse::Mouse;
use position::{self, Depth, Dimensions, HorizontalAlign, Position, VerticalAlign};
use ui::{UiId, Ui};
use widget::Kind;

/// Represents the state of the Toggle widget.
#[derive(Clone, Copy, Debug, PartialEq)]
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

widget_fns!(Toggle, State, Kind::Toggle(State::Normal));

/// Check the current state of the button.
fn get_new_state(is_over: bool,
                 prev: State,
                 mouse: Mouse) -> State {
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

/// A pressable widget for toggling the state of a bool. Like the button widget, it's reaction is
/// triggered upon release and will return the new bool state. Note that the toggle will not
/// mutate the bool for you, you should do this yourself within the react closure.
pub struct Toggle<'a, F> {
    pos: Position,
    dim: Dimensions,
    maybe_h_align: Option<HorizontalAlign>,
    maybe_v_align: Option<VerticalAlign>,
    depth: Depth,
    maybe_react: Option<F>,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
    value: bool,
}

impl<'a, F> Toggle<'a, F> {

    /// Construct a new Toggle widget.
    pub fn new(value: bool) -> Toggle<'a, F> {
        Toggle {
            pos: Position::default(),
            dim: [64.0, 64.0],
            maybe_h_align: None,
            maybe_v_align: None,
            depth: 0.0,
            maybe_react: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
            value: value,
        }
    }

    /// Set the reaction for the Toggle. It will be triggered upon release of the button.
    pub fn react(mut self, reaction: F) -> Self {
        self.maybe_react = Some(reaction);
        self
    }

    /// After building the Toggle, use this method to set its current state into the given `Ui`.
    /// It will use this state for rendering the next time `ui.draw(graphics)` is called.
    pub fn set<C>(mut self, ui_id: UiId, ui: &mut Ui<C>)
        where
            F: FnMut(bool),
    {
        use utils::is_over_rect;

        let state = *get_state(ui, ui_id);
        let dim = self.dim;
        let h_align = self.maybe_h_align.unwrap_or(ui.theme.h_align);
        let v_align = self.maybe_v_align.unwrap_or(ui.theme.v_align);
        let xy = ui.get_xy(self.pos, self.dim, h_align, v_align);
        let mouse = ui.get_mouse_state(ui_id);
        let is_over = is_over_rect(xy, mouse.xy, self.dim);
        let new_state = get_new_state(is_over, state, mouse);

        // React.
        if let (true, State::Clicked, State::Highlighted) = (is_over, state, new_state) {
            if let Some(ref mut react) = self.maybe_react { react(!self.value) }
        }

        let draw_new_element_condition = true;

        // Only update the Element if the state has changed.
        let maybe_new_element = if draw_new_element_condition {
            use elmesque::form::{collage, rect, text};

            // Construct the frame and pressable forms.
            let frame_w = self.maybe_frame.unwrap_or(ui.theme.frame_width);
            let frame_color = self.maybe_frame_color.unwrap_or(ui.theme.frame_color);
            let (inner_w, inner_h) = (dim[0] - frame_w * 2.0, dim[1] - frame_w * 2.0);
            let frame_form = rect(dim[0], dim[1]).filled(frame_color);
            let color = self.maybe_color.unwrap_or(ui.theme.shape_color);
            let color = new_state.color(if self.value { color } else { color.with_luminance(0.1) });
            let pressable_form = rect(inner_w, inner_h).filled(color);

            // Construct the label's Form.
            let maybe_label_form = self.maybe_label.map(|label_text| {
                use elmesque::text::Text;
                let text_color = self.maybe_label_color.unwrap_or(ui.theme.label_color);
                let size = self.maybe_label_font_size.unwrap_or(ui.theme.font_size_medium);
                text(Text::from_string(label_text.to_string()).color(text_color).height(size as f64))
                    .shift(xy[0].floor(), xy[1].floor())
            });

            // Chain the Forms and shift them into position.
            let form_chain = Some(frame_form).into_iter()
                .chain(Some(pressable_form).into_iter())
                .map(|form| form.shift(xy[0], xy[1]))
                .chain(maybe_label_form.into_iter());

            // Collect the Forms into a renderable Element.
            let element = collage(dim[0] as i32, dim[1] as i32, form_chain.collect());

            Some(element)

        } else { None };

        // Store the button's new state in the Ui.
        ui.update_widget(ui_id, Kind::Toggle(new_state), xy, self.depth, maybe_new_element);

    }

}

impl<'a, F> Colorable for Toggle<'a, F> {
    fn color(mut self, color: Color) -> Self {
        self.maybe_color = Some(color);
        self
    }
}

impl<'a, F> Frameable for Toggle<'a, F> {
    fn frame(mut self, width: f64) -> Self {
        self.maybe_frame = Some(width);
        self
    }
    fn frame_color(mut self, color: Color) -> Self {
        self.maybe_frame_color = Some(color);
        self
    }
}

impl<'a, F> Labelable<'a> for Toggle<'a, F> {
    fn label(mut self, text: &'a str) -> Self {
        self.maybe_label = Some(text);
        self
    }

    fn label_color(mut self, color: Color) -> Self {
        self.maybe_label_color = Some(color);
        self
    }

    fn label_font_size(mut self, size: FontSize) -> Self {
        self.maybe_label_font_size = Some(size);
        self
    }
}

impl<'a, F> position::Positionable for Toggle<'a, F> {
    fn position(mut self, pos: Position) -> Self {
        self.pos = pos;
        self
    }
    #[inline]
    fn horizontal_align(self, h_align: HorizontalAlign) -> Self {
        Toggle { maybe_h_align: Some(h_align), ..self }
    }
    #[inline]
    fn vertical_align(self, v_align: VerticalAlign) -> Self {
        Toggle { maybe_v_align: Some(v_align), ..self }
    }
}

impl<'a, F> position::Sizeable for Toggle<'a, F> {
    #[inline]
    fn width(self, w: f64) -> Self {
        let h = self.dim[1];
        Toggle { dim: [w, h], ..self }
    }
    #[inline]
    fn height(self, h: f64) -> Self {
        let w = self.dim[0];
        Toggle { dim: [w, h], ..self }
    }
}

