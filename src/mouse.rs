
use point::Point;

/// Represents the current state of a mouse button.
#[derive(Debug, Clone, Copy)]
pub enum ButtonState {
    Up,
    Down,
}

/// Represents the current state of the Mouse.
#[derive(Copy, Clone)]
pub struct Mouse {
    pub xy: Point,
    pub left: ButtonState,
    pub middle: ButtonState,
    pub right: ButtonState,
}

impl Mouse {
    /// Constructor for a Mouse struct.
    pub fn new(xy: Point,
               left: ButtonState,
               middle: ButtonState,
               right: ButtonState) -> Mouse {
        Mouse { xy: xy, left: left, middle: middle, right: right }
    }
}
