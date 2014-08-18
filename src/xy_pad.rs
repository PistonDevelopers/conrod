
use piston::{
    RenderArgs,
};
use opengl_graphics::Gl;
use graphics::{
    AddColor,
    AddLine,
    AddSquareBorder,
    Context,
    Draw,
};
use point::Point;
use color::Color;
use frame::{
    Framing,
    Frame,
    NoFrame,
};
use label;
use label::{
    NoLabel,
    Label,
    FontSize,
    Labeling,
};
use rectangle;
use std::num::{
    from_f64,
    pow,
};
use ui_context::{
    UIID,
    UIContext,
};
use mouse_state::{
    MouseState,
    Up,
    Down,
};
use utils::{
    map,
    percentage,
};
use widget::XYPad;

/// Represents the state of the Button widget.
#[deriving(Show, PartialEq)]
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

widget_fns!(XYPad, State, XYPad(Normal))

/// Draw the xy_pad. When successfully pressed,
/// the given `callback` closure will be called
/// with the xy coordinates as params.
//#[inline]
pub fn draw
    <X: Num + Copy + ToPrimitive + FromPrimitive + ToString,
     Y: Num + Copy + ToPrimitive + FromPrimitive + ToString>
           (args: &RenderArgs,
            gl: &mut Gl,
            uic: &mut UIContext,
            ui_id: UIID,
            pos: Point<f64>,
            width: f64,
            height: f64,
            font_size: FontSize,
            frame: Framing,
            color: Color,
            label: Labeling,
            x: X, min_x: X, max_x: X,
            y: Y, min_y: Y, max_y: Y,
            callback: |X, Y|) {

    let state = get_state(uic, ui_id);
    let mouse = uic.get_mouse_state();

    // Rect.
    let frame_w = match frame { Frame(w, _) => w, NoFrame => 0.0 };
    let frame_w2 = frame_w * 2.0;
    let is_over = rectangle::is_over(pos, mouse.pos, width, height);
    let rect_state = get_new_state(is_over, state, mouse).as_rectangle_state();
    rectangle::draw(args, gl, rect_state, pos, width, height, frame, color);

    // Don't include the rect frame for the interactive pad.
    let pad_pos = pos + Point::new(frame_w, frame_w, 0.0);
    let (pad_w, pad_h) = (width - frame_w2, height - frame_w2);
    let is_over_pad = rectangle::is_over(pad_pos, mouse.pos, pad_w, pad_h);
    let new_state = get_new_state(is_over_pad, state, mouse);

    // Crosshair.
    let (new_x, new_y, vert_x, hori_y) = match (is_over_pad, new_state) {
        (false, _) | (true, Normal) | (true, Highlighted) => {
            (x, y,
             pad_pos.x + map(x.to_f64().unwrap(),
                             min_x.to_f64().unwrap(),
                             max_x.to_f64().unwrap(), pad_w, 0.0),
             pad_pos.y + map(y.to_f64().unwrap(),
                             min_y.to_f64().unwrap(),
                             max_y.to_f64().unwrap(), pad_h, 0.0))
        },
        (_, Clicked) => {
            (from_f64(
                map(mouse.pos.x - pos.x, pad_w, 0.0,
                    min_x.to_f64().unwrap(), max_x.to_f64().unwrap())
             ).unwrap(),
             from_f64(
                map(mouse.pos.y - pos.y, pad_h, 0.0,
                    min_y.to_f64().unwrap(), max_y.to_f64().unwrap())
             ).unwrap(),
             mouse.pos.x, mouse.pos.y)
        }
    };
    draw_crosshair(args, gl, pad_pos,
                   vert_x, hori_y,
                   pad_w, pad_h,
                   color.plain_contrast());

    // Label.
    match label {
        NoLabel => (),
        Label(l_text, l_size, l_color) => {
            let l_w = label::width(uic, l_size, l_text);
            let l_pos = Point::new(pad_pos.x + (pad_w - l_w) / 2.0,
                                   pad_pos.y + (pad_h - l_size as f64) / 2.0,
                                   0.0);
            label::draw(args, gl, uic, l_pos, l_size, l_color, l_text);
        },
    };

    // xy value string.
    let x_string = val_to_string(new_x, max_x, max_x - min_x, width as uint);
    let y_string = val_to_string(new_y, max_y, max_y - min_y, height as uint);
    let xy_string = x_string.append(", ").append(y_string.as_slice());
    let xy_string_w = label::width(uic, font_size, xy_string.as_slice());
    let x_perc = percentage(new_x, min_x, max_x);
    let y_perc = percentage(new_y, min_y, max_y);
    let xy_string_pos = {
        // If crosshair is in bottom left.
        if      x_perc <= 0.5 && y_perc <= 0.5 {
            Point::new(vert_x - xy_string_w, hori_y - font_size as f64, 0.0)
        }
        // If crosshair is in bottom right.
        else if x_perc >  0.5 && y_perc <= 0.5 {
            Point::new(vert_x, hori_y - font_size as f64, 0.0)
        }
        // If crosshair is in top left.
        else if x_perc <= 0.5 && y_perc >  0.5 {
            Point::new(vert_x - xy_string_w, hori_y, 0.0)
        }
        // If crosshair is in top right.
        else {
            Point::new(vert_x, hori_y, 0.0)
        }
    };
    label::draw(args, gl, uic, xy_string_pos, font_size,
                color.plain_contrast(), xy_string.as_slice());

    // Set the new state.
    set_state(uic, ui_id, new_state);

    // Callback if value is changed or the pad is clicked/released.
    if x != new_x || y != new_y { callback(new_x, new_y) }
    else {
        match (state, new_state) {
            (Highlighted, Clicked)
            | (Clicked, Highlighted) => callback(new_x, new_y),
            _ => (),
        }
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

/// Draw the crosshair.
fn draw_crosshair(args: &RenderArgs,
                  gl: &mut Gl,
                  pos: Point<f64>,
                  vert_x: f64, hori_y: f64,
                  pad_w: f64, pad_h: f64,
                  color: Color) {
    let context = &Context::abs(args.width as f64, args.height as f64);
    let (r, g, b, a) = color.as_tuple();
    context
        .line(vert_x, pos.y, vert_x, pos.y + pad_h)
        .square_border_width(1.0)
        .rgba(r, g, b, a)
        .draw(gl);
    context
        .line(pos.x, hori_y, pos.x + pad_w, hori_y)
        .square_border_width(1.0)
        .rgba(r, g, b, a)
        .draw(gl);
}

/// Get a suitable string from the value, its max and the pixel range.
fn val_to_string<T: ToString + ToPrimitive>
                (val: T, max: T, val_rng: T, pixel_range: uint) -> String {
    let mut s = val.to_string();
    let decimal = s.as_slice().chars().position(|ch| ch == '.');
    match decimal {
        None => s,
        Some(idx) => {
            // Find the minimum string length by determing
            // what power of ten both the max and range are.
            let val_rng_f = val_rng.to_f64().unwrap();
            let max_f = max.to_f64().unwrap();
            let mut n = 0f64;
            let mut pow_ten = 0f64;
            while pow_ten < val_rng_f || pow_ten < max_f {
                pow_ten = (10f64).powf(n);
                n += 1.0
            }
            let min_string_len = n as uint;

            // Find out how many pixels there are to actually use
            // and judge a reasonable precision from this.
            let mut n = 0u;
            while pow(10u, n) < pixel_range { n += 1u }
            let precision = n - 1u;

            // Truncate the length to the pixel precision as
            // long as this doesn't cause it to be smaller
            // than the necessary decimal place.
            let truncate_len = {
                if precision >= min_string_len { precision }
                else { min_string_len }
            };
            if truncate_len - 1u == idx { s.truncate(truncate_len + 1u) }
            else { s.truncate(truncate_len) }
            s
        }
    }
}

