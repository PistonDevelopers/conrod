
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
use widget::Button;

/// Represents the state of the Button widget.
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

widget_fns!(Button, State, Button(Normal))

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
pub struct ButtonContext<'a> {
    uic: &'a mut UIContext,
    ui_id: UIID,
    pos: Point<f64>,
    width: f64,
    height: f64,
    maybe_color: Option<Color>,
    maybe_frame: Option<(f64, Color)>,
    maybe_label: Option<(&'a str, u32, Color)>,
    maybe_callback: Option<||:'a>,
}

pub trait ButtonBuilder<'a> {
    /// A button builder method to be implemented by the UIContext.
    fn button(&'a mut self, ui_id: UIID) -> ButtonContext<'a>;
}

impl<'a> ButtonBuilder<'a> for UIContext {

    /// Create a button context to be built upon.
    fn button(&'a mut self, ui_id: UIID) -> ButtonContext<'a> {
        ButtonContext {
            uic: self,
            ui_id: ui_id,
            pos: Point::new(0.0, 0.0, 0.0),
            width: 64.0,
            height: 64.0,
            maybe_callback: None,
            maybe_color: None,
            maybe_frame: None,
            maybe_label: None,
        }
    }

}

impl_colorable!(ButtonContext)
impl_frameable!(ButtonContext)
impl_labelable!(ButtonContext)
impl_positionable!(ButtonContext)
impl_shapeable!(ButtonContext)

impl<'a> ::callback::Callable<||:'a> for ButtonContext<'a> {
    #[inline]
    fn callback(self, callback: ||:'a) -> ButtonContext<'a> {
        ButtonContext { maybe_callback: Some(callback), ..self }
    }
}

impl<'a> ::draw::Drawable for ButtonContext<'a> {
    fn draw(&mut self, gl: &mut Gl) {

        let state = *get_state(self.uic, self.ui_id);
        let mouse = self.uic.get_mouse_state();
        let is_over = rectangle::is_over(self.pos, mouse.pos, self.width, self.height);
        let new_state = get_new_state(is_over, state, mouse);
        set_state(self.uic, self.ui_id, new_state);

        // Callback.
        match (is_over, state, new_state) {
            (true, Clicked, Highlighted) => match self.maybe_callback {
                Some(ref mut callback) => (*callback)(), None => (),
            }, _ => (),
        }

        // Draw.
        let rect_state = new_state.as_rectangle_state();
        let color = self.maybe_color.unwrap_or(::std::default::Default::default());
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

