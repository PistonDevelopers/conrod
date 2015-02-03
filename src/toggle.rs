
use color::Color;
use dimensions::Dimensions;
use mouse::Mouse;
use point::Point;
use rectangle;
use graphics::BackEnd;
use graphics::character::CharacterCache;
use ui_context::{
    Id,
    UIID,
    UiContext,
};
use widget::{ DefaultWidgetState, Widget };
use Callback;
use FrameColor;
use FrameWidth;
use LabelText;
use LabelColor;
use LabelFontSize;
use Position;
use Size;

/// Represents the state of the Toggle widget.
#[derive(PartialEq, Clone, Copy)]
pub enum State {
    Normal,
    Highlighted,
    Clicked,
}

impl State {
    /// Return the associated Rectangle state.
    fn as_rectangle_state(&self) -> rectangle::State {
        match self {
            &State::Normal => rectangle::State::Normal,
            &State::Highlighted => rectangle::State::Highlighted,
            &State::Clicked => rectangle::State::Clicked,
        }
    }
}

widget_fns!(Toggle, State, Widget::Toggle(State::Normal));

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

/// A context on which the builder pattern can be implemented.
pub struct Toggle<'a, F> {
    ui_id: UIID,
    pos: Point,
    dim: Dimensions,
    maybe_callback: Option<F>,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
    value: bool,
}

impl<'a, F> Toggle<'a, F> {

    /// Create a toggle context to be built upon.
    pub fn new(ui_id: UIID, value: bool) -> Toggle<'a, F> {
        Toggle {
            ui_id: ui_id,
            pos: [0.0, 0.0],
            dim: [64.0, 64.0],
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
            maybe_label_color: None,
            maybe_label_font_size: None,
            value: value,
        }
    }

}

quack! {
    toggle: Toggle['a, F]
    get:
        fn () -> Size [] { Size(toggle.dim) }
        fn () -> DefaultWidgetState [] {
            DefaultWidgetState(Widget::Toggle(State::Normal))
        }
        fn () -> Id [] { Id(toggle.ui_id) }
    set:
        fn (val: Color) [] { toggle.maybe_color = Some(val) }
        fn (val: Callback<F>) [where F: FnMut(bool) + 'a] {
            toggle.maybe_callback = Some(val.0)
        }
        fn (val: FrameColor) [] { toggle.maybe_frame_color = Some(val.0) }
        fn (val: FrameWidth) [] { toggle.maybe_frame = Some(val.0) }
        fn (val: LabelText<'a>) [] { toggle.maybe_label = Some(val.0) }
        fn (val: LabelColor) [] { toggle.maybe_label_color = Some(val.0) }
        fn (val: LabelFontSize) [] {
            toggle.maybe_label_font_size = Some(val.0)
        }
        fn (val: Position) [] { toggle.pos = val.0 }
        fn (val: Size) [] { toggle.dim = val.0 }
    action:
}

impl<'a, F> ::draw::Drawable for Toggle<'a, F> where F: FnMut(bool) + 'a {
    fn draw<B, C>(&mut self, uic: &mut UiContext<C>, graphics: &mut B)
        where
            B: BackEnd<Texture = <C as CharacterCache>::Texture>,
            C: CharacterCache
    {
        let color = self.maybe_color.unwrap_or(uic.theme.shape_color);
        let color = match self.value {
            true => color,
            false => color * Color::new(0.1, 0.1, 0.1, 1.0)
        };
        let state = *get_state(uic, self.ui_id);
        let mouse = uic.get_mouse_state();
        let is_over = rectangle::is_over(self.pos, mouse.pos, self.dim);
        let new_state = get_new_state(is_over, state, mouse);
        let rect_state = new_state.as_rectangle_state();
        match self.maybe_callback {
            Some(ref mut callback) => {
                match (is_over, state, new_state) {
                    (true, State::Clicked, State::Highlighted) =>
                        (*callback)(match self.value { true => false, false => true }),
                    _ => (),
                }
            }, None => (),
        }
        let frame_w = self.maybe_frame.unwrap_or(uic.theme.frame_width);
        let maybe_frame = match frame_w > 0.0 {
            true => Some((frame_w, self.maybe_frame_color.unwrap_or(uic.theme.frame_color))),
            false => None,
        };
        match self.maybe_label {
            None => {
                rectangle::draw(
                    uic.win_w, uic.win_h, graphics, rect_state, self.pos,
                    self.dim, maybe_frame, color
                )
            },
            Some(text) => {
                let text_color = self.maybe_label_color.unwrap_or(uic.theme.label_color);
                let size = self.maybe_label_font_size.unwrap_or(uic.theme.font_size_medium);
                rectangle::draw_with_centered_label(
                    uic.win_w, uic.win_h, graphics, uic, rect_state,
                    self.pos, self.dim, maybe_frame, color,
                    text, size, text_color
                )
            },
        }

        set_state(uic, self.ui_id, Widget::Toggle(new_state), self.pos, self.dim);

    }
}
