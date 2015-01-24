
use color::Color;
use internal::Dimensions;
use mouse::Mouse;
use opengl_graphics::Gl;
use point::Point;
use rectangle;
use ui_context::{
    UIID,
    UiContext,
};
use widget::Widget::Toggle;

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

widget_fns!(Toggle, State, Toggle(State::Normal));

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
pub struct ToggleContext<'a> {
    uic: &'a mut UiContext,
    ui_id: UIID,
    pos: Point,
    dim: Dimensions,
    maybe_callback: Option<Box<FnMut(bool) + 'a>>,
    maybe_color: Option<Color>,
    maybe_frame: Option<f64>,
    maybe_frame_color: Option<Color>,
    maybe_label: Option<&'a str>,
    maybe_label_color: Option<Color>,
    maybe_label_font_size: Option<u32>,
    value: bool,
}

pub trait ToggleBuilder<'a> {
    /// A builder method to be implemented by the UiContext.
    fn toggle(&'a mut self, ui_id: UIID, value: bool) -> ToggleContext<'a>;
}

impl<'a> ToggleBuilder<'a> for UiContext {

    /// Create a toggle context to be built upon.
    fn toggle(&'a mut self, ui_id: UIID, value: bool) -> ToggleContext<'a> {
        ToggleContext {
            uic: self,
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

impl_callable!(ToggleContext, FnMut(bool),);
impl_colorable!(ToggleContext,);
impl_frameable!(ToggleContext,);
impl_labelable!(ToggleContext,);
impl_positionable!(ToggleContext,);
impl_shapeable!(ToggleContext,);

impl<'a> ::draw::Drawable for ToggleContext<'a> {
    fn draw(&mut self, graphics: &mut Gl) {
        let color = self.maybe_color.unwrap_or(self.uic.theme.shape_color);
        let color = match self.value {
            true => color,
            false => color * Color::new(0.1, 0.1, 0.1, 1.0)
        };
        let state = *get_state(self.uic, self.ui_id);
        let mouse = self.uic.get_mouse_state();
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
        let frame_w = self.maybe_frame.unwrap_or(self.uic.theme.frame_width);
        let maybe_frame = match frame_w > 0.0 {
            true => Some((frame_w, self.maybe_frame_color.unwrap_or(self.uic.theme.frame_color))),
            false => None,
        };
        match self.maybe_label {
            None => {
                rectangle::draw(
                    self.uic.win_w, self.uic.win_h, graphics, rect_state, self.pos,
                    self.dim, maybe_frame, color
                )
            },
            Some(text) => {
                let text_color = self.maybe_label_color.unwrap_or(self.uic.theme.label_color);
                let size = self.maybe_label_font_size.unwrap_or(self.uic.theme.font_size_medium);
                rectangle::draw_with_centered_label(
                    self.uic.win_w, self.uic.win_h, graphics, self.uic, rect_state,
                    self.pos, self.dim, maybe_frame, color,
                    text, size, text_color
                )
            },
        }

        set_state(self.uic, self.ui_id, new_state, self.pos, self.dim);

    }
}
