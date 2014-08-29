
use color::Color;
use frame::{
    Framing,
    Frame,
    NoFrame,
};
use label;
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
use piston::RenderArgs;
use point::Point;
use rectangle;
use ui_context::{
    UIID,
    UIContext,
};
use utils::{
    clamp,
    percentage,
    value_from_perc,
};
use widget::Slider;

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

widget_fns!(Slider, State, Slider(Normal))

/// A generic slider user-interface widget. It will
/// automatically convert itself between horizontal
/// and vertical depending on the dimensions given.
pub fn draw<T: Num + Copy + FromPrimitive + ToPrimitive>
    (args: &RenderArgs,
     gl: &mut Gl,
     uic: &mut UIContext,
     ui_id: UIID,
     pos: Point<f64>,
     width: f64,
     height: f64,
     frame: Framing,
     color: Color,
     label: Labeling,
     value: T,
     min: T,
     max: T,
     callback: |T|) {

    let state = *get_state(uic, ui_id);
    let mouse = uic.get_mouse_state();
    let is_over = rectangle::is_over(pos, mouse.pos, width, height);
    let new_state = get_new_state(is_over, state, mouse);
    let rect_state = new_state.as_rectangle_state();
    let (frame_w, frame_c) = match frame {
        Frame(frame_w, frame_c) => (frame_w, frame_c),
        NoFrame => (0.0, Color::black()),
    };
    rectangle::draw(args, gl, rect_state, pos, width, height, NoFrame, frame_c);
    let is_horizontal = width > height;
    let (r_pos, r_width, r_height, new_val) = match is_horizontal {
        true => horizontal(pos, width, height, frame_w,
                           value, min, max, mouse, is_over, state, new_state),
        false => vertical(pos, width, height, frame_w,
                          value, min, max, mouse, is_over, state, new_state),
    };
    rectangle::draw(args, gl, rect_state, r_pos, r_width, r_height, NoFrame, color);
    match label {
        NoLabel => (),
        Label(text, size, text_color) => {
            let l_pos = match is_horizontal {
                true => {
                    let x = r_pos.x + (r_height - size as f64) / 2.0;
                    let y = r_pos.y + (r_height - size as f64) / 2.0;
                    Point::new(x, y, 0f64)
                },
                false => {
                    let x = r_pos.x + (r_width - label::width(uic, size, text.as_slice())) / 2.0;
                    let y = r_pos.y + r_height - r_width - frame_w;
                    Point::new(x, y, 0f64)
                },
            };
            label::draw(args, gl, uic, l_pos, size, text_color, text.as_slice());
        }
    }
    set_state(uic, ui_id, new_state);
    if value != new_val || match (state, new_state) {
        (Highlighted, Clicked) | (Clicked, Highlighted) => true,
        _ => false,
    } { callback(new_val) };
}

/// Horizontal slider.
fn horizontal<T: Num + Copy + FromPrimitive + ToPrimitive>
             (pos: Point<f64>,
              width: f64,
              height: f64,
              frame_w: f64,
              value: T,
              min: T,
              max: T,
              mouse: MouseState,
              is_over: bool,
              state: State,
              new_state: State) -> (Point<f64>, f64, f64, T) {
    let p = pos + Point::new(frame_w, frame_w, 0f64);
    let max_width = width - (frame_w * 2f64);
    let w = match (is_over, state, new_state) {
        (true, Highlighted, Clicked) | (true, Clicked, Clicked) | (false, Clicked, Clicked) => {
            clamp(mouse.pos.x - p.x, 0f64, max_width)
        },
        _ => clamp(percentage(value, min, max) as f64 * max_width, 0f64, max_width),
    };
    let h = height - (frame_w * 2f64);
    let v = value_from_perc((w / max_width) as f32, min, max);
    (p, w, h, v)
}

/// Vertical slider.
fn vertical<T: Num + Copy + FromPrimitive + ToPrimitive>
            (pos: Point<f64>,
             width: f64,
             height: f64,
             frame_w: f64,
             value: T,
             min: T,
             max: T,
             mouse: MouseState,
             is_over: bool,
             state: State,
             new_state: State) -> (Point<f64>, f64, f64, T) {
    let corner = pos + Point::new(frame_w, frame_w, 0f64);
    let max_height = height - (frame_w * 2f64);
    let y_max = corner.y + max_height;
    let (h, p) = match (is_over, state, new_state) {
        (true, Highlighted, Clicked) | (true, Clicked, Clicked) | (false, Clicked, Clicked) => {
            let p = Point::new(corner.x, clamp(mouse.pos.y, corner.y, y_max), 0f64);
            let h = clamp(max_height - (p.y - corner.y), 0f64, max_height);
            (h, p)
        },
        _ => {
            let h = clamp(percentage(value, min, max) as f64 * max_height, 0f64, max_height);
            let p = Point::new(corner.x, corner.y + max_height - h, 0f64);
            (h, p)
        },
    };
    let w = width - (frame_w * 2f64);
    let v = value_from_perc((h / max_height) as f32, min, max);
    (p, w, h, v)
}

/// Check the current state of the slider.
fn get_new_state(is_over: bool,
                 prev: State,
                 mouse: MouseState) -> State {
    match (is_over, prev, mouse) {
        (true, Normal, MouseState { left: Down, .. }) => Normal,
        (true, _, MouseState { left: Down, .. }) => Clicked,
        (true, _, MouseState { left: Up, .. }) => Highlighted,
        (false, Clicked, MouseState { left: Down, .. }) => Clicked,
        _ => Normal,
    }
}

