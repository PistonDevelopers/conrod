
use opengl_graphics::Gl;
use piston::{
    RenderArgs,
};
use color::Color;
use point::Point;
use rectangle;
use rectangle::RectangleState;
use widget::{
    Widget,
    Button,
};
use ui_context::{
    UIContext,
    MouseState,
    Up,
    Down,
};

widget_state!(ButtonState, ButtonState {
    Normal -> 0,
    Highlighted -> 1,
    Clicked -> 2
})

impl ButtonState {
    /// Return the associated Rectangle state.
    fn as_rectangle_state(&self) -> RectangleState {
        match self {
            &Normal => rectangle::Normal,
            &Highlighted => rectangle::Highlighted,
            &Clicked => rectangle::Clicked,
        }
    }
}

/// Draw the button. When successfully pressed,
/// the given `event` function will be called.
pub fn draw(args: &RenderArgs,
            gl: &mut Gl,
            uic: &mut UIContext,
            ui_id: uint,
            pos: Point<f64>,
            width: f64,
            height: f64,
            border: f64,
            color: Color,
            event: ||) {
    let state = get_state(uic, ui_id);
    let mouse = uic.get_mouse_state();
    let is_over = rectangle::is_over(pos, mouse.pos, width, height);
    let new_state = check_state(is_over, state, mouse);
    let rect_state = new_state.as_rectangle_state();
    rectangle::draw(args, gl, rect_state, pos, width, height, border, color);
    set_state(uic, ui_id, new_state);
    match (is_over, state, new_state) {
        (true, Clicked, Highlighted) => event(),
        _ => (),
    }
}

/// Get a reference to the widget associated with the given UIID.
fn get_widget(uic: &mut UIContext, ui_id: uint) -> &mut Widget {
    uic.get_widget(ui_id, Button(Normal))
}

/// Get the current ButtonState for the widget.
fn get_state(uic: &mut UIContext, ui_id: uint) -> ButtonState {
    match *get_widget(uic, ui_id) {
        Button(state) => state,
        _ => fail!("The Widget variant returned by UIContext is different to the requested."),
    }
}

/// Set the state for the widget in the UIContext.
fn set_state(uic: &mut UIContext, ui_id: uint, new_state: ButtonState) {
    match *get_widget(uic, ui_id) {
        Button(ref mut state) => { *state = new_state; },
        _ => fail!("The Widget variant returned by UIContext is different to the requested."),
    }
}

/// Check the current state of the button.
fn check_state(is_over: bool,
               prev: ButtonState,
               mouse: MouseState) -> ButtonState {
    match (is_over, prev, mouse) {
        (true, _, MouseState { left: Down, .. }) => Clicked,
        (true, _, MouseState { left: Up, .. }) => Highlighted,
        _ => Normal,
    }
}

