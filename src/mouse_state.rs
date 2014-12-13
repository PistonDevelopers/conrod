
use point::Point;

/// Represents the current state of a mouse button.
#[deriving(Show, Clone, Copy)]
pub enum MouseButtonState {
    Up,
    Down,
}

/// Represents the current state of the Mouse.
#[deriving(Copy)]
pub struct MouseState {
    pub pos: Point,
    pub left: MouseButtonState,
    pub middle: MouseButtonState,
    pub right: MouseButtonState,
}

impl MouseState {
    /// Constructor for a MouseState struct.
    pub fn new(pos: Point,
               left: MouseButtonState,
               middle: MouseButtonState,
               right: MouseButtonState) -> MouseState {
        MouseState { pos: pos, left: left, middle: middle, right: right }
    }
}

