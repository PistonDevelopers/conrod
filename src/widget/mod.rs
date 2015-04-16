
use elmesque::Element;

pub mod button;
pub mod drop_down_list;
pub mod envelope_editor;
pub mod label;
pub mod matrix;
pub mod number_dialer;
pub mod slider;
pub mod text_box;
pub mod toggle;
pub mod xy_pad;

/// Represents some Widget type.
#[derive(Clone)]
pub struct Widget {
    pub kind: Kind,
    pub placing: Placing,
    pub maybe_element: Option<Element>,
}

impl Widget {

    /// Construct an empty Widget for a vacant widget position within the Ui.
    pub fn empty() -> Widget {
        Widget {
            kind: Kind::NoWidget,
            placing: Placing::NoPlace,
            maybe_element: None,
        }
    }

    /// Construct a Widget from a given kind.
    pub fn new(kind: Kind) -> Widget {
        Widget {
            kind: kind,
            placing: Placing::NoPlace,
            maybe_element: None,
        }
    }

}

/// Algebraic widget type for storing in ui_context
/// and for ease of state-matching.
#[derive(Copy, Clone)]
pub enum Kind {
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

impl Kind {
    pub fn matches(&self, other: &Kind) -> bool {
        match (self, other) {
            (&Kind::NoWidget, &Kind::NoWidget) => true,
            (&Kind::Button(_), &Kind::Button(_)) => true,
            (&Kind::DropDownList(_), &Kind::DropDownList(_)) => true,
            (&Kind::EnvelopeEditor(_), &Kind::EnvelopeEditor(_)) => true,
            (&Kind::NumberDialer(_), &Kind::NumberDialer(_)) => true,
            (&Kind::Slider(_), &Kind::Slider(_)) => true,
            (&Kind::TextBox(_), &Kind::TextBox(_)) => true,
            (&Kind::Toggle(_), &Kind::Toggle(_)) => true,
            (&Kind::XYPad(_), &Kind::XYPad(_)) => true,
            _ => false
        }
    }
}

/// Represents the placement of the widget including
/// x / y position, width and height.
#[derive(Clone, Copy)]
pub enum Placing {
    Place(f64, f64, f64, f64), // (x, y, w, h)
    NoPlace,
}

impl Placing {
    pub fn down(&self, padding: f64) -> (f64, f64) {
        match self {
            &Placing::Place(x, y, _, h) => (x, y + h + padding),
            &Placing::NoPlace => (0.0, 0.0),
        }
    }
    pub fn up(&self, padding: f64) -> (f64, f64) {
        match self {
            &Placing::Place(x, y, _, _) => (x, y - padding),
            &Placing::NoPlace => (0.0, 0.0),
        }
    }
    pub fn left(&self, padding: f64) -> (f64, f64) {
        match self {
            &Placing::Place(x, y, _, _) => (x - padding, y),
            &Placing::NoPlace => (0.0, 0.0),
        }
    }
    pub fn right(&self, padding: f64) -> (f64, f64) {
        match self {
            &Placing::Place(x, y, w, _) => (x + w + padding, y),
            &Placing::NoPlace => (0.0, 0.0),
        }
    }
}

