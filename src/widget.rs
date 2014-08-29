
use button;
use drop_down_list;
use envelope_editor;
use number_dialer;
use slider;
use text_box;
use toggle;
use xy_pad;

/// Algebraic widget type for storing in ui_context
/// and for ease of state-matching.
pub enum Widget {
    Button(button::State),
    DropDownList(drop_down_list::State),
    EnvelopeEditor(envelope_editor::State),
    NumberDialer(number_dialer::State),
    Slider(slider::State),
    TextBox(text_box::State),
    Toggle(toggle::State),
    XYPad(xy_pad::State),
}

