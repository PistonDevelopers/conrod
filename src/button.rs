use quack::{ Get, Set };
use color::Color;
use opengl_graphics::Gl;
use mouse::Mouse;
use rectangle;
use ui_context::{
    UIID,
    UiContext,
};
use widget::Widget;
use internal;
use Dimensions;
use FontSize;
use Frame;
use MaybeColor;
use Label;
use Position;
use Text;

/// Represents the state of the Button widget.
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

widget_fns!(Button, State, Widget::Button(State::Normal));

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

/////////////////////////////// NEW DESIGN /////////////////////////////////////

/// A button.
pub struct Button<'a> {
    pos: internal::Point,
    dim: internal::Dimensions,
    maybe_color: Option<internal::Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<Label<'a>>,
}

impl<'a> Button<'a> {
    pub fn new() -> Self {
        Button {
            pos: [0.0, 0.0],
            dim: [64.0, 64.0],
            maybe_color: None,
            maybe_frame: None,
            maybe_frame_color: None,
            maybe_label: None,
        }
    }

    pub fn draw(
        &self, ui_id: UIID,
        mut maybe_callback: Option<Box<FnMut()>>,
        uic: &mut UiContext,
        graphics: &mut Gl
    ) {

        let state = *get_state(uic, ui_id);
        let mouse = uic.get_mouse_state();
        let is_over = rectangle::is_over(self.pos, mouse.pos, self.dim);
        let new_state = get_new_state(is_over, state, mouse);

        // Callback.
        match (is_over, state, new_state) {
            (true, State::Clicked, State::Highlighted) => match maybe_callback {
                Some(ref mut callback) => (*callback)(), None => (),
            }, _ => (),
        }

        // Draw.
        let rect_state = new_state.as_rectangle_state();
        let color = self.maybe_color.unwrap_or(uic.theme.shape_color.0);
        let frame_w = self.maybe_frame.unwrap_or(uic.theme.frame_width);
        let maybe_frame = match frame_w > 0.0 {
            true => Some((frame_w, self.maybe_frame_color.unwrap_or(uic.theme.frame_color))),
            false => None,
        };
        match self.maybe_label {
            None => {
                rectangle::draw(
                    uic.win_w, uic.win_h, graphics, rect_state, self.pos,
                    self.dim, maybe_frame, Color(color)
                )
            },
            Some(label) => {
                use vecmath::vec2_add as add;

                let Text(text) = label.get();
                let MaybeColor(maybe_color) = label.get();
                let size: FontSize = label.get();
                let Position(label_pos) = label.get();

                let text_color = maybe_color.unwrap_or(uic.theme.label_color.0);
                let size = size.size(&uic.theme);
                let pos = add(self.pos, label_pos);
                rectangle::draw_with_centered_label(
                    uic.win_w, uic.win_h, graphics, uic, rect_state,
                    pos, self.dim, maybe_frame, Color(color),
                    text, size, Color(text_color)
                )
            },
        }

        set_state(uic, ui_id, new_state, self.pos, self.dim);

    }
}

quack! {
    button: Button['a]
    get:
        fn () -> Position { Position(button.pos) }
        fn () -> MaybeColor { MaybeColor(button.maybe_color) }
        fn () -> Dimensions { Dimensions(button.dim) }
    set:
        fn (val: Position) { button.pos = val.0 }
        fn (val: Color) { button.maybe_color = Some(val.0) }
        fn (val: Dimensions) { button.dim = val.0 }
        fn (val: Text<'a>) {
            let text = val.0;
            button.maybe_label = match button.maybe_label {
                None => Some(Label::new(text)),
                Some(x) => Some(x.set(Text(text)))
            };
        }
        fn (val: Label<'a>) { button.maybe_label = Some(val) }
        fn (val: Frame) { button.maybe_frame = Some(val.0) }
    action:
}
