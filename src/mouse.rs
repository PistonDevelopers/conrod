//!
//! A module for describing Mouse state.
//!
//! The `Ui` will continuously maintain the latest Mouse state, necessary for widget logic.
//!

pub use input::MouseButton;
pub use graphics::math::Scalar;
use position::Point;
use time::{SteadyTime, Duration};
use std::collections::HashMap;

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
    /// Movements less than this threshold will not be considered drags
    pub drag_distance_threshold: Scalar,
    last_button_down_events: [Option<MouseButtonDown>; 9],
}


/// Used for simplified mouse event handling. Most widgets can probably
/// just use these events
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SimpleMouseEvent {
    Click(MouseClick),
    Drag(MouseDragEvent),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseClick {
    mouse_button: MouseButton,
    position: Point
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseDragEvent {
    mouse_button: MouseButton,
    button_down_position: Point,
    current_position: Point
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct MouseButtonDown {
    time: SteadyTime,
    position: Point
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
            drag_distance_threshold: 2.0,
            last_button_down_events: [None; 9],
        }
    }

    /// Return the mouse state with its position relative to the given position.
    pub fn relative_to(self, xy: Point) -> Mouse {
        Mouse { xy: ::vecmath::vec2_sub(self.xy, xy), ..self }
    }

    /// sets the new position of the mouse
    pub fn move_to(&mut self, xy: Point) {
        self.xy = xy;
    }

    /// Sends the mouse a button down event
    pub fn button_down(&mut self, button: MouseButton) {
        use input::MouseButton::*;

        let button_down = MouseButtonDown {
            time: SteadyTime::now(),
            position: self.xy.clone()
        };
        self.set_last_button_down_time(button, Some(button_down));

        let button_state = match button {
            Left => &mut self.left,
            Right => &mut self.right,
            Middle => &mut self.middle,
            _ => &mut self.unknown
        };
        button_state.position = ButtonPosition::Down;
        button_state.was_just_pressed = true;
    }

    pub fn button_up(&mut self, button: MouseButton) {
        use input::MouseButton::*;

        self.set_last_button_down_time(button, None);
        let button_state = match button {
            Left => &mut self.left,
            Right => &mut self.right,
            Middle => &mut self.middle,
            _ => &mut self.unknown
        };
        button_state.position = ButtonPosition::Up;
        button_state.was_just_released = true;
    }

    pub fn was_clicked(&self, button: MouseButton) -> bool {
        use input::MouseButton::*;
        let button_state = match button {
            Left => self.left,
            Right => self.right,
            Middle => self.middle,
            _ => self.unknown
        };
        button_state.was_just_released
    }

    pub fn get_simple_event(&self) -> Option<SimpleMouseEvent> {
        use input::MouseButton::{Left, Right, Middle};
        None
    }

    fn get_last_button_down(&self, button: MouseButton) -> Option<MouseButtonDown> {
        let index: u32 = button.into();
        self.last_button_down_events[index as usize]
    }

    fn set_last_button_down_time(&mut self, button: MouseButton, maybe_last_down: Option<MouseButtonDown>) {
        let index: u32 = button.into();
        self.last_button_down_events[index as usize] = maybe_last_down;
    }

}

#[test]
fn move_to_sets_new_mouse_position() {
    let mut mouse = Mouse::new();

    let new_position = [2.0, 5.0];
    mouse.move_to(new_position);
    assert_eq!(new_position, mouse.xy);
}

#[test]
fn button_down_sets_last_button_down_info_and_button_up_removes_it() {
    use input::MouseButton::Left;

    let mut mouse = Mouse::new();
    assert!(mouse.get_last_button_down(Left).is_none());

    mouse.button_down(Left);
    assert!(mouse.get_last_button_down(Left).is_some());

    mouse.button_up(Left);
    assert!(mouse.get_last_button_down(Left).is_none());
}

#[test]
fn button_down_sets_button_state_to_down() {
    let mut mouse = Mouse::new();
    mouse.left.position = ButtonPosition::Up;

    mouse.button_down(MouseButton::Left);

    assert_eq!(ButtonPosition::Down, mouse.left.position);
}

#[test]
fn button_up_sets_button_state_to_up() {
    let mut mouse = Mouse::new();
    mouse.left.position = ButtonPosition::Down;

    mouse.button_up(MouseButton::Left);
    assert_eq!(ButtonPosition::Up, mouse.left.position);
}

#[test]
fn get_simple_event_returns_click_if_button_goes_down_then_up_in_same_position() {
    use input::MouseButton::Right;
    let mut mouse = Mouse::new();
    mouse.button_down(Right);
    mouse.button_up(Right);
    let expected_event = SimpleMouseEvent::Click(MouseClick{
        mouse_button: Right,
        position: mouse.xy.clone()
    });

    let actual_event = mouse.get_simple_event();
    assert!(actual_event.is_some());
    assert_eq!(expected_event, actual_event.unwrap());
}

#[test]
fn get_simple_event_returns_drag_event_if_mouse_was_dragged() {
    use input::MouseButton::Left;
    let mut mouse = Mouse::new();
    let new_position = [0.0, mouse.drag_distance_threshold + 1.0];
    mouse.button_down(Left);
    mouse.move_to(new_position);
    mouse.button_up(Left);

    let expected_event = SimpleMouseEvent::Drag(MouseDragEvent{
        mouse_button: Left,
        button_down_position: [0.0, 0.0],
        current_position: new_position,
    });

    let actual_event = mouse.get_simple_event();
    assert!(actual_event.is_some());
    assert_eq!(expected_event, actual_event.unwrap());
}
