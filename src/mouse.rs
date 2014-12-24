
use point::Point;

/// Represents the current state of a mouse button.
#[deriving(Show, Clone, Copy)]
pub enum ButtonState {
    Up,
    Down,
}

/// Represents the current state of the Mouse.
#[deriving(Copy)]
pub struct Mouse {
    pub pos: Point,
    pub left: ButtonState,
    pub middle: ButtonState,
    pub right: ButtonState,
}

impl Mouse {
    /// Constructor for a Mouse struct.
    pub fn new(pos: Point,
               left: ButtonState,
               middle: ButtonState,
               right: ButtonState) -> Mouse {
        Mouse { pos: pos, left: left, middle: middle, right: right }
    }
}
