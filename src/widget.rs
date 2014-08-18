
use button;
use drop_down_list;
use number_dialer;
use slider;
use toggle;
use xy_pad;

/// Algebraic widget type for storing in ui_context
/// and for ease of state-matching.
pub enum Widget {
    Button(button::State),
    DropDownList(drop_down_list::State),
    NumberDialer(number_dialer::State),
    Slider(slider::State),
    Toggle(toggle::State),
    XYPad(xy_pad::State),
}

