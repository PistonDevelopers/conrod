
use piston::{
    RenderArgs,
};
use label;
use label::{
    IsLabel,
    Label,
    NoLabel,
};
use rectangle;
use rectangle::RectangleState;
use point::Point;
use color::Color;
use utils::clamp;
use opengl_graphics::Gl;
use ui_context::{
    UIContext,
    MouseState,
    Up,
    Down,
};
use std::num::from_f32;
use widget::{
    Widget,
    Slider,
};

widget_state!(SliderState, SliderState {
    Normal -> 0,
    Highlighted -> 0,
    Clicked -> 2
})

impl SliderState {
    /// Return the associated Rectangle state.
    fn as_rectangle_state(&self) -> RectangleState {
        match self {
            &Normal => rectangle::Normal,
            &Highlighted => rectangle::Highlighted,
            &Clicked => rectangle::Clicked,
        }
    }
}

/// A generic slider user-interface widget. It will
/// automatically convert itself between horizontal
/// and vertical depending on the dimensions given.
pub fn draw<T: Num + Copy + FromPrimitive + ToPrimitive>
    (args: &RenderArgs,
     gl: &mut Gl,
     uic: &mut UIContext,
     ui_id: uint,
     pos: Point<f64>,
     width: f64,
     height: f64,
     border: f64,
     color: Color,
     label: IsLabel,
     value: T,
     min: T,
     max: T,
     event: |T|) {
    let state = get_state(uic, ui_id);
    let mouse = uic.get_mouse_state();
    let is_horizontal = width > height;
    let is_over = rectangle::is_over(pos, mouse.pos, width, height);
    let new_state = check_state(is_over, state, mouse);
    let rect_state = new_state.as_rectangle_state();
    rectangle::draw(args, gl, rect_state, pos, width, height, 0f64, Color::black());
    let (r_pos, r_width, r_height, new_val) = match is_horizontal {
        true => horizontal(pos, width, height, border, value, min, max, mouse, is_over, state, new_state),
        false => vertical(pos, width, height, border, value, min, max, mouse, is_over, state, new_state),
    };
    rectangle::draw(args, gl, rect_state, r_pos, r_width, r_height, 0f64, color);
    match label {
        NoLabel => (),
        Label(text, size, text_color) => {
            let l_pos = match is_horizontal {
                true => {
                    let x = r_pos.x + border;
                    let y = r_pos.y + (r_height - size as f64) / 2.0;
                    Point::new(x, y, 0f64)
                },
                false => {
                    let x = r_pos.x + (r_width - label::get_text_width(uic, size, text.as_slice())) / 2.0;
                    let y = r_pos.y + r_height - size as f64 - border;
                    Point::new(x, y, 0f64)
                },
            };
            label::draw(args, gl, uic, l_pos, size, text_color, text.as_slice());
        }
    }
    set_state(uic, ui_id, new_state);
    event(new_val);
}

/// Horizontal slider.
fn horizontal<T: Num + Copy + FromPrimitive + ToPrimitive>
             (pos: Point<f64>,
              width: f64,
              height: f64,
              border: f64,
              value: T,
              min: T,
              max: T,
              mouse: MouseState,
              is_over: bool,
              state: SliderState,
              new_state: SliderState) -> (Point<f64>, f64, f64, T) {
    let p = pos + Point::new(border, border, 0f64);
    let max_width = width - (border * 2f64);
    let w = match (is_over, state, new_state) {
        (true, Highlighted, Clicked) | (true, Clicked, Clicked)
            => clamp(mouse.pos.x - p.x, 0f64, max_width),
        _ => clamp(get_percentage(value, min, max) as f64 * max_width, 0f64, max_width),
    };
    let h = height - (border * 2f64);
    let v = get_value((w / max_width) as f32, min, max);
    (p, w, h, v)
}

/// Vertical slider.
fn vertical<T: Num + Copy + FromPrimitive + ToPrimitive>
            (pos: Point<f64>,
             width: f64,
             height: f64,
             border: f64,
             value: T,
             min: T,
             max: T,
             mouse: MouseState,
             is_over: bool,
             state: SliderState,
             new_state: SliderState) -> (Point<f64>, f64, f64, T) {
    let corner = pos + Point::new(border, border, 0f64);
    let max_height = height - (border * 2f64);
    let y_max = corner.y + max_height;
    let (h, p) = match (is_over, state, new_state) {
        (true, Highlighted, Clicked) | (true, Clicked, Clicked) => {
            let p = Point::new(corner.x, clamp(mouse.pos.y, corner.y, y_max), 0f64);
            let h = clamp(max_height - (p.y - corner.y), 0f64, max_height);
            (h, p)
        },
        _ => {
            let h = clamp(get_percentage(value, min, max) as f64 * max_height, 0f64, max_height);
            let p = Point::new(corner.x, corner.y + max_height - h, 0f64);
            (h, p)
        },
    };
    let w = width - (border * 2f64);
    let v = get_value((h / max_height) as f32, min, max);
    (p, w, h, v)
}

/// Get a reference to the widget associated with the given UIID.
fn get_widget(uic: &mut UIContext, ui_id: uint) -> &mut Widget {
    uic.get_widget(ui_id, Slider(Normal))
}

/// Get the current SliderState for the widget.
fn get_state(uic: &mut UIContext, ui_id: uint) -> SliderState {
    match *get_widget(uic, ui_id) {
        Slider(state) => state,
        _ => fail!("The Widget variant returned by UIContext is different to the requested."),
    }
}

/// Set the state for the widget in the UIContext.
fn set_state(uic: &mut UIContext, ui_id: uint, new_state: SliderState) {
    match *get_widget(uic, ui_id) {
        Slider(ref mut state) => { *state = new_state; },
        _ => fail!("The Widget variant returned by UIContext is different to the requested."),
    }
}

/// Get value percentage between max and min.
fn get_percentage<T: Num + Copy + FromPrimitive + ToPrimitive>
    (value: T, min: T, max: T) -> f32 {
    let v = value.to_f32().unwrap();
    let mn = min.to_f32().unwrap();
    let mx = max.to_f32().unwrap();
    (v - mn) / (mx - mn)
}

/// Adjust the value to the given percentage.
fn get_value<T: Num + Copy + FromPrimitive + ToPrimitive>
    (perc: f32, min: T, max: T) -> T {
    from_f32::<T>((max - min).to_f32().unwrap() * perc).unwrap() + min
}

/// Check the current state of the slider.
fn check_state(is_over: bool,
               prev: SliderState,
               mouse: MouseState) -> SliderState {
    match (is_over, prev, mouse) {
        (true, _, MouseState { left: Down, .. }) => Clicked,
        (true, _, MouseState { left: Up, .. }) => Highlighted,
        _ => Normal,
    }
}

