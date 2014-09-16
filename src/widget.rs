
use button;
use drop_down_list;
use envelope_editor;
use number_dialer;
use slider;
use text_box;
use toggle;
use xy_pad;

/// Represents the placement of the widget including
/// x / y position, width and height.
#[deriving(Clone)]
pub struct Placing {
    pub x: f64, 
    pub y: f64, 
    pub w: f64, 
    pub h: f64,
}



impl Placing {
    pub fn down(&self, padding: f64) -> (f64, f64) {
        (self.x, self.y + self.h + padding)
    }

    pub fn up(&self, padding: f64) -> (f64, f64) {
        (self.x, self.y - padding)
    }

    pub fn left(&self, padding: f64) -> (f64, f64) {
        (self.x - padding, self.y)
    }

    pub fn right(&self, padding: f64) -> (f64, f64) {
        (self.x + self.w + padding, self.y)
    }
}

/// Algebraic widget type for storing in ui_context
/// and for ease of state-matching.
#[deriving(Clone)]
pub enum Widget {
    NoWidget,
    Button(button::State),
    DropDownList(drop_down_list::State),
    EnvelopeEditor(envelope_editor::State),
    NumberDialer(number_dialer::State),
    Slider(slider::State),
    TextBox(text_box::State),
    Toggle(toggle::State),
    XYPad(xy_pad::State),
}

