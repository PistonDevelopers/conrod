
use std::from_str::FromStr;
use opengl_graphics::Gl;
use piston::RenderArgs;
use color::Color;
use point::Point;
use rectangle;
use rectangle::RectangleState;
use widget::NumberDialer;
use utils::clamp;
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
    FontSize,
    IsLabel,
    NoLabel,
    Label,
};

/// Represents the state of the Button widget.
#[deriving(PartialEq)]
pub enum NumberDialerState {
    Normal,
    Highlighted,
    Clicked,
}

impl NumberDialerState {
    /// Return the associated Rectangle state.
    fn as_rectangle_state(&self) -> RectangleState {
        match self {
            &Normal => rectangle::Normal,
            &Highlighted => rectangle::Highlighted,
            &Clicked => rectangle::Clicked,
        }
    }
}

widget_fns!(NumberDialer, NumberDialerState, NumberDialer(Normal))

/// Draw the number_dialer. When successfully pressed,
/// or if the value is changed, the given `callback`
/// function will be called.
pub fn draw<T: Num + Copy + Primitive + FromPrimitive + ToPrimitive + ToString + FromStr>
    (args: &RenderArgs,
     gl: &mut Gl,
     uic: &mut UIContext,
     ui_id: UIID,
     pos: Point<f64>,
     font_size: FontSize,
     color: Color,
     label: IsLabel,
     value: T,
     min: T,
     max: T,
     precision: u8,
     callback: |T|) {
    let val = clamp(value, min, max);
    let state = get_state(uic, ui_id);
    let mouse = uic.get_mouse_state();
    let val_string = create_string(val, precision);
    //println!("{}", string.as_slice());
    



    
    // TODO
    // determine rect dimensions.
    // draw rect.
    // draw string.
    // if being interacted with, determine which glyph captures the mouse.
    // determine new value by comparing previous state with new state.
}

/// Create the string to be drawn from the given values
/// and precision. Combine this with the label string if
/// one is given.
fn create_string<T: ToString>(val: T, precision: u8) -> String {
    let mut val_string = val.to_string();
    match (val_string.as_slice().chars().position(|ch| ch == '.'), precision) {
        (None, 0u8) => val_string,
        (None, _) => {
            val_string.push_char('.');
            val_string.grow(precision as uint, '0');
            val_string
        },
        (Some(idx), 0u8) => {
            val_string.truncate(idx);
            val_string
        },
        (Some(idx), _) => {
            let (len, desired_len) = (val_string.len(), idx + precision as uint + 1u);
            match len.cmp(&desired_len) {
                Greater => val_string.truncate(desired_len),
                Equal => (),
                Less => val_string.grow(desired_len - len, '0'),
            }
            val_string
        },
    }
}

///// Return the total width of string glyphs.
//fn string_dimensions(font_size: FontSize,
//                     label: IsLabel,
//                     val_string: String) -> (f64, f64) {
//    let label_string = match label {
//        NoLabel => String::new(),
//        Label(ref text, _, _) => text.to_string().append(": "),
//    };
//}

/// Return the dimensions of a character slot for the given font_size.
fn character_slot_dimensions(font_size: FontSize) -> (f64, f64) {
    (font_size as f64, (font_size as f64 * 1.5).round())
}

