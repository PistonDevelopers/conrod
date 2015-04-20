
use position::Point;

/// Represents the current state of a mouse button.
#[derive(Clone, Copy, Debug)]
pub enum ButtonState {
    Up,
    Down,
}

/// Represents the current state of the Mouse.
#[derive(Copy, Clone, Debug)]
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

    /// Return the mouse state with its position relative to the given position.
    pub fn relative_to(self, xy: Point) -> Mouse {
        Mouse { xy: ::vecmath::vec2_sub(self.xy, xy), ..self }
    }

}
