
use std::from_str::FromStr;
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
    NumberDialer,
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
use label;
use label::{
    FontSize,
    IsLabel,
    NoLabel,
    Label,
};
use utils::clamp;

widget_state!(NumberDialerState, NumberDialerState {
    Normal -> 0,
    Highlighted -> 1,
    Clicked -> 2
})

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
    let string = create_string(&label, val, precision);
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
fn create_string<T: ToString>(label: &IsLabel, val: T, precision: u8) -> String {
    let label_string = match *label {
        NoLabel => String::new(),
        Label(ref text, _, _) => text.to_string().append(": "),
    };
    let mut val_string = val.to_string();
    match (val_string.as_slice().chars().position(|ch| ch == '.'), precision) {
        (None, 0u8) => label_string.append(val_string.as_slice()),
        (None, _) => {
            val_string.push_char('.');
            val_string.grow(precision as uint, '0');
            label_string.append(val_string.as_slice())
        },
        (Some(idx), 0u8) => {
            val_string.truncate(idx);
            label_string.append(val_string.as_slice())
        },
        (Some(idx), _) => {
            let (len, desired_len) = (val_string.len(), idx + precision as uint + 1u);
            match len.cmp(&desired_len) {
                Greater => val_string.truncate(desired_len),
                Equal => (),
                Less => val_string.grow(desired_len - len, '0'),
            }
            label_string.append(val_string.as_slice())
        },
    }
}

/// Return a default Widget variant.
fn default() -> Widget { NumberDialer(Normal) }

/// Get a reference to the widget associated with the given UIID.
fn get_widget(uic: &mut UIContext, ui_id: UIID) -> &mut Widget {
    uic.get_widget(ui_id, default())
}

/// Get the current SliderState for the widget.
fn get_state(uic: &mut UIContext, ui_id: UIID) -> NumberDialerState {
    match *get_widget(uic, ui_id) {
        NumberDialer(state) => state,
        _ => fail!("The Widget variant returned by UIContext is different to the requested."),
    }
}

/// Set the state for the widget in the UIContext.
fn set_state(uic: &mut UIContext, ui_id: UIID, new_state: NumberDialerState) {
    match *get_widget(uic, ui_id) {
        NumberDialer(ref mut state) => { *state = new_state; },
        _ => fail!("The Widget variant returned by UIContext is different to the requested."),
    }
}



