
use point::Point;

/// Represents the current state of a mouse button.
#[deriving(Show, Clone)]
pub enum MouseButtonState {
    Up,
    Down,
}

/// Represents the current state of the Mouse.
#[deriving(Show, Clone)]
pub struct MouseState {
    pub pos: Point<f64>,
    pub left: MouseButtonState,
    pub middle: MouseButtonState,
    pub right: MouseButtonState,
}

impl MouseState {
    /// Constructor for a MouseState struct.
    pub fn new(pos: Point<f64>,
               left: MouseButtonState,
               middle: MouseButtonState,
               right: MouseButtonState) -> MouseState {
        MouseState { pos: pos, left: left, middle: middle, right: right }
    }
}

