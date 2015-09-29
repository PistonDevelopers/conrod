//! 
//! A module for describing Mouse state.
//!
//! The `Ui` will continuously maintain the latest Mouse state, necessary for widget logic.
//!

use position::Point;

/// The current state of a Mouse button.
#[derive(Copy, Clone, Debug)]
pub struct ButtonState {
    /// The button has been pressed since the last update.
    pub was_just_pressed: bool,
    /// The button has been released since the last update.
    pub was_just_released: bool,
    /// The current position of the button.
    pub position: ButtonPosition,
}

/// Represents the current state of a mouse button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonPosition {
    /// The mouse button is currently up.
    Up,
    /// The mouse button is currently down (pressed).
    Down,
}

/// Represents the current state of the Mouse.
#[derive(Copy, Clone, Debug)]
pub struct Mouse {
    /// Position of the mouse cursor.
    pub xy: Point,
    /// Left mouse button state.
    pub left: ButtonState,
    /// Middle mouse button state.
    pub middle: ButtonState,
    /// Right mouse button state.
    pub right: ButtonState,
    /// Unknown button state.
    pub unknown: ButtonState,
    /// Amount that the mouse has scrolled since the last render.
    pub scroll: Scroll,
}

/// The amount of scrolling that has occurred since the last render event.
#[derive(Copy, Clone, Debug)]
pub struct Scroll {
    /// Scrolling across the x axis.
    pub x: f64,
    /// Scrolling across the y axis.
    pub y: f64,
}


impl ButtonState {

    /// Constructor for a default ButtonState.
    pub fn new() -> ButtonState {
        ButtonState {
            was_just_released: false,
            was_just_pressed: false,
            position: ButtonPosition::Up,
        }
    }

    /// Reset the `was_just_released` and `was_just_pressed` flags.
    pub fn reset_pressed_and_released(&mut self) {
        self.was_just_released = false;
        self.was_just_pressed = false;
    }

}


impl Mouse {

    /// Constructor for a default Mouse struct.
    pub fn new() -> Mouse {
        Mouse {
            xy: [0.0, 0.0],
            left: ButtonState::new(),
            middle: ButtonState::new(),
            right: ButtonState::new(),
            unknown: ButtonState::new(),
            scroll: Scroll { x: 0.0, y: 0.0 },
        }
    }

    /// Return the mouse state with its position relative to the given position.
    pub fn relative_to(self, xy: Point) -> Mouse {
        Mouse { xy: ::vecmath::vec2_sub(self.xy, xy), ..self }
    }

}

