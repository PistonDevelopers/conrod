
use color::Color;
use frame::Framing;
use label::{
    Labeling,
    NoLabel,
    Label,
};
use mouse_state::{
    MouseState,
    Up,
    Down,
};
use opengl_graphics::Gl;
use piston::RenderArgs;
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

/// Draw the button. When successfully pressed,
/// the given `callback` function will be called.
pub fn draw(args: &RenderArgs,
            gl: &mut Gl,
            uic: &mut UIContext,
            ui_id: UIID,
            pos: Point<f64>,
            width: f64,
            height: f64,
            frame: Framing,
            color: Color,
            label: Labeling,
            callback: ||) {
    let state = *get_state(uic, ui_id);
    let mouse = uic.get_mouse_state();
    let is_over = rectangle::is_over(pos, mouse.pos, width, height);
    let new_state = get_new_state(is_over, state, mouse);
    let rect_state = new_state.as_rectangle_state();
    match label {
        NoLabel => {
            rectangle::draw(args, gl, rect_state, pos, width, height, frame, color)
        },
        Label(text, size, text_color) => {
            rectangle::draw_with_centered_label(args, gl, uic, rect_state,
                                                pos, width, height, frame, color,
                                                text, size, text_color)
        },
    }
    set_state(uic, ui_id, new_state);
    match (is_over, state, new_state) {
        (true, Clicked, Highlighted) => callback(),
        _ => (),
    }
}

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

