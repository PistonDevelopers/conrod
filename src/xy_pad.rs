
use color::Color;
use frame::{
    Framing,
    Frame,
    NoFrame,
};
use graphics::{
    AddColor,
    AddLine,
    AddSquareBorder,
    Context,
    Draw,
};
use label;
use label::{
    NoLabel,
    Label,
    FontSize,
    Labeling,
};
use mouse_state::{
    MouseState,
    Up,
    Down,
};
use opengl_graphics::Gl;
use point::Point;
use rectangle;
use rectangle::{
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
};
use std::num::from_f64;
use ui_context::{
    UIID,
    UIContext,
};
use utils::{
    clamp,
    map_range,
    val_to_string,
};
use widget::XYPad;

/// Represents the state of the xy_pad widget.
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
pub fn draw
    <X: Num + Copy + ToPrimitive + FromPrimitive + ToString,
     Y: Num + Copy + ToPrimitive + FromPrimitive + ToString>
           (gl: &mut Gl,
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

    let state = *get_state(uic, ui_id);
    let mouse = uic.get_mouse_state();

    // Rect.
    let frame_w = match frame { Frame(w, _) => w, NoFrame => 0.0 };
    let frame_w2 = frame_w * 2.0;
    let is_over = rectangle::is_over(pos, mouse.pos, width, height);
    let rect_state = get_new_state(is_over, state, mouse).as_rectangle_state();
    rectangle::draw(uic.win_w, uic.win_h, gl, rect_state, pos,
                    width, height, frame, color);

    // Don't include the rect frame for the interactive pad.
    let pad_pos = pos + Point::new(frame_w, frame_w, 0.0);
    let (pad_w, pad_h) = (width - frame_w2, height - frame_w2);
    let is_over_pad = rectangle::is_over(pad_pos, mouse.pos, pad_w, pad_h);
    let new_state = get_new_state(is_over_pad, state, mouse);

    // Crosshair.
    let (new_x, new_y, vert_x, hori_y) = match (is_over_pad, new_state) {
        (_, Normal) | (_, Highlighted) => {
            (
                x,
                y,
                pad_pos.x + map_range(x.to_f64().unwrap(),
                                      min_x.to_f64().unwrap(),
                                      max_x.to_f64().unwrap(), pad_w, 0.0),
                pad_pos.y + map_range(y.to_f64().unwrap(),
                                      min_y.to_f64().unwrap(),
                                      max_y.to_f64().unwrap(), pad_h, 0.0)
            )
        },
        (_, Clicked) => {
            let temp_x = clamp(mouse.pos.x, pad_pos.x, pad_pos.x + pad_w);
            let temp_y = clamp(mouse.pos.y, pad_pos.y, pad_pos.y + pad_h);
            (
                from_f64(
                    map_range(temp_x - pos.x, pad_w, 0.0,
                              min_x.to_f64().unwrap(), max_x.to_f64().unwrap())
                ).unwrap(),
                from_f64(
                    map_range(temp_y - pos.y, pad_h, 0.0,
                              min_y.to_f64().unwrap(), max_y.to_f64().unwrap())
                ).unwrap(),
                temp_x,
                temp_y
            )
        }
    };

    // Crosshair.
    draw_crosshair(uic.win_w, uic.win_h, gl, pad_pos,
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
            label::draw(gl, uic, l_pos, l_size, l_color, l_text);
        },
    };

    // xy value string.
    let x_string = val_to_string(new_x, max_x, max_x - min_x, width as uint);
    let y_string = val_to_string(new_y, max_y, max_y - min_y, height as uint);
    let xy_string = x_string.append(", ").append(y_string.as_slice());
    let xy_string_w = label::width(uic, font_size, xy_string.as_slice());
    //let x_perc = percentage(new_x, min_x, max_x);
    //let y_perc = percentage(new_y, min_y, max_y);
    let xy_string_pos = {
        match rectangle::corner(pad_pos, Point::new(vert_x, hori_y, 0.0), pad_w, pad_h) {
            TopLeft => Point::new(vert_x, hori_y, 0.0),
            TopRight => Point::new(vert_x - xy_string_w, hori_y, 0.0),
            BottomLeft => Point::new(vert_x, hori_y - font_size as f64, 0.0),
            BottomRight => Point::new(vert_x - xy_string_w, hori_y - font_size as f64, 0.0),
        }
    };
    label::draw(gl, uic, xy_string_pos, font_size, color.plain_contrast(), xy_string.as_slice());

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
fn draw_crosshair(win_w: f64,
                  win_h: f64,
                  gl: &mut Gl,
                  pos: Point<f64>,
                  vert_x: f64, hori_y: f64,
                  pad_w: f64, pad_h: f64,
                  color: Color) {
    let context = &Context::abs(win_w, win_h);
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

