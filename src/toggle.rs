
use color::Color;
use label::{
    Labeling,
    Label,
    NoLabel,
};
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
#[deriving(PartialEq)]
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
    is_over: bool,
    state: State,
    new_state: State,
    pos: Point<f64>,
    width: f64,
    height: f64,
    maybe_color: Option<Color>,
    maybe_frame: Option<(f64, Color)>,
    maybe_label: Option<(&'a str, u32, Color)>,
    value: bool,
}

pub trait ToggleBuilder<'a> {
    /// A builder method to be implemented by the UIContext.
    fn toggle(&'a mut self, ui_id: UIID, value: bool,
              x: f64, y: f64, width: f64, height: f64) -> ToggleContext<'a>;
}

impl<'a> ToggleBuilder<'a> for UIContext {

    /// Create a toggle context to be built upon.
    fn toggle(&'a mut self, ui_id: UIID, value: bool,
              x: f64, y: f64, width: f64, height: f64) -> ToggleContext<'a> {
        let pos = Point::new(x, y, 0.0);
        let state = *get_state(self, ui_id);
        let mouse = self.get_mouse_state();
        let is_over = rectangle::is_over(pos, mouse.pos, width, height);
        let new_state = get_new_state(is_over, state, mouse);
        set_state(self, ui_id, new_state);
        ToggleContext {
            is_over: is_over,
            state: state,
            new_state: new_state,
            pos: pos,
            width: width,
            height: height,
            maybe_color: None,
            maybe_frame: None,
            maybe_label: None,
            uic: self,
            value: value,
        }
    }

}

impl_colorable!(ToggleContext)
impl_frameable!(ToggleContext)
impl_labelable!(ToggleContext)
impl_positionable!(ToggleContext)

impl<'a> ::callback::Callable<|bool|:'a> for ToggleContext<'a> {
    #[inline]
    fn callback(self, callback: |bool|) -> ToggleContext<'a> {
        match (self.is_over, self.state, self.new_state) {
            (true, Clicked, Highlighted) =>
                callback(match self.value { true => false, false => true }),
            _ => (),
        }
        self
    }
}

impl<'a> ::draw::Drawable for ToggleContext<'a> {
    fn draw(&mut self, gl: &mut Gl) {
        let rect_state = self.new_state.as_rectangle_state();
        let color: Color = match self.maybe_color {
            None => ::std::default::Default::default(),
            Some(color) => color,
        };
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

