
use color::Color;
use mouse_state::{
    MouseState,
    Up,
    Down,
};
use opengl_graphics::Gl;
use point::Point;
use rectangle;
use ui_context::{
    UIID,
    UIContext,
};
use widget::Toggle;

/// Represents the state of the Toggle widget.
#[deriving(PartialEq, Clone)]
pub enum State {
    Normal,
    Highlighted,
    Clicked,
}

impl State {
    /// Return the associated Rectangle state.
    fn as_rectangle_state(&self) -> rectangle::State {
        match self {
            &Normal => rectangle::Normal,
            &Highlighted => rectangle::Highlighted,
            &Clicked => rectangle::Clicked,
        }
    }
}

widget_fns!(Toggle, State, Toggle(Normal))

/// Check the current state of the button.
fn get_new_state(is_over: bool,
                 prev: State,
                 mouse: MouseState) -> State {
    match (is_over, prev, mouse.left) {
        (true, Normal, Down) => Normal,
        (true, _, Down) => Clicked,
        (true, _, Up) => Highlighted,
        (false, Clicked, Down) => Clicked,
        _ => Normal,
    }
}

/// A context on which the builder pattern can be implemented.
pub struct ToggleContext<'a> {
    uic: &'a mut UIContext,
    ui_id: UIID,
    pos: Point<f64>,
    width: f64,
    height: f64,
    maybe_callback: Option<|bool|:'a>,
    maybe_color: Option<Color>,
    maybe_frame: Option<(f64, Color)>,
    maybe_label: Option<(&'a str, u32, Color)>,
    value: bool,
}

pub trait ToggleBuilder<'a> {
    /// A builder method to be implemented by the UIContext.
    fn toggle(&'a mut self, ui_id: UIID, value: bool) -> ToggleContext<'a>;
}

impl<'a> ToggleBuilder<'a> for UIContext {

    /// Create a toggle context to be built upon.
    fn toggle(&'a mut self, ui_id: UIID, value: bool) -> ToggleContext<'a> {
        ToggleContext {
            uic: self,
            ui_id: ui_id,
            pos: Point::new(0.0, 0.0, 0.0),
            width: 64.0,
            height: 64.0,
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_label: None,
            value: value,
        }
    }

}

impl_colorable!(ToggleContext)
impl_frameable!(ToggleContext)
impl_labelable!(ToggleContext)
impl_positionable!(ToggleContext)
impl_shapeable!(ToggleContext)

impl<'a> ::callback::Callable<|bool|:'a> for ToggleContext<'a> {
    #[inline]
    fn callback(self, callback: |bool|:'a) -> ToggleContext<'a> {
        ToggleContext { maybe_callback: Some(callback), ..self }
    }
}

impl<'a> ::draw::Drawable for ToggleContext<'a> {
    fn draw(&mut self, gl: &mut Gl) {
        let color = self.maybe_color.unwrap_or(::std::default::Default::default());
        let color = match self.value {
            true => color,
            false => color * Color::new(0.1, 0.1, 0.1, 1.0)
        };
        let state = *get_state(self.uic, self.ui_id);
        let mouse = self.uic.get_mouse_state();
        let is_over = rectangle::is_over(self.pos, mouse.pos, self.width, self.height);
        let new_state = get_new_state(is_over, state, mouse);
        set_state(self.uic, self.ui_id, new_state);
        let rect_state = new_state.as_rectangle_state();
        match self.maybe_callback {
            Some(ref mut callback) => {
                match (is_over, state, new_state) {
                    (true, Clicked, Highlighted) =>
                        (*callback)(match self.value { true => false, false => true }),
                    _ => (),
                }
            }, None => (),
        }
        match self.maybe_label {
            None => {
                rectangle::draw(
                    self.uic.win_w, self.uic.win_h, gl, rect_state, self.pos,
                    self.width, self.height, self.maybe_frame, color
                )
            },
            Some((text, size, text_color)) => {
                rectangle::draw_with_centered_label(
                    self.uic.win_w, self.uic.win_h, gl, self.uic, rect_state,
                    self.pos, self.width, self.height, self.maybe_frame, color,
                    text, size, text_color
                )
            },
        }
    }
}

