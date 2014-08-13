
use opengl_graphics::Gl;
use piston::RenderArgs;
use color::Color;
use point::Point;
use rectangle;
use rectangle::RectangleState;
use widget::Toggle;
use ui_context::{
    UIID,
    UIContext,
};
use mouse_state::{
    MouseState,
    Up,
    Down,
};
use label;
use label::{
    Labeling,
    Label,
    NoLabel,
};
use frame::Framing;

/// Represents the state of the Button widget.
#[deriving(PartialEq)]
pub enum State {
    Normal,
    Highlighted,
    Clicked,
}

impl State {
    /// Return the associated Rectangle state.
    fn as_rectangle_state(&self) -> RectangleState {
        match self {
            &Normal => rectangle::Normal,
            &Highlighted => rectangle::Highlighted,
            &Clicked => rectangle::Clicked,
        }
    }
}

widget_fns!(Toggle, State, Toggle(Normal))

/// Draw a Toggle widget. If the toggle
/// is switched, call `event` with the
/// new state.
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
            value: bool,
            callback: |bool|) {
    let state = get_state(uic, ui_id);
    let mouse = uic.get_mouse_state();
    let is_over = rectangle::is_over(pos, mouse.pos, width, height);
    let new_state = check_state(is_over, state, mouse);
    let rect_state = new_state.as_rectangle_state();
    let rect_color = match value {
        true => color,
        false => Color::new(
                color.r() * 0.1f32, 
                color.g() * 0.1f32, 
                color.b() * 0.1f32, 
                color.a()
            ),
    };
    rectangle::draw(args, gl, rect_state, pos, width, height, frame, rect_color);
    match label {
        NoLabel => (),
        Label(text, size, text_color) => {
            let t_w = label::width(uic, size, text);
            let x = pos.x + (width - t_w) / 2.0;
            let y = pos.y + (height - size as f64) / 2.0;
            let l_pos = Point::new(x, y, 0.0);
            label::draw(args, gl, uic, l_pos, size, text_color, text);
        },
    }
    set_state(uic, ui_id, new_state);
    match (is_over, state, new_state) {
        (true, Clicked, Highlighted) => callback(match value { true => false, false => true }),
        _ => (),
    }
}

/// Check the current state of the button.
fn check_state(is_over: bool,
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

