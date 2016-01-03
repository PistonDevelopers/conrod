//!
//! A module for describing Mouse state.
//!
//! The `Ui` will continuously maintain the latest Mouse state, necessary for widget logic.
//!

pub mod simple_events;
mod button_map;

pub use self::button_map::{ButtonMap, NUM_MOUSE_BUTTONS};
pub use input::MouseButton;

use graphics::math::Scalar;
use self::simple_events::*;
use position::Point;
use time::{SteadyTime};

/// The current state of a Mouse button.
#[derive(Copy, Clone, Debug)]
pub struct ButtonState {
    /// The button has been pressed since the last update.
    pub was_just_pressed: bool,
    /// The button has been released since the last update.
    pub was_just_released: bool,
    /// The current position of the button.
    pub position: ButtonPosition,
    /// SimpleMouseEvent for events corresponding to this button.
    pub event: Option<SimpleMouseEvent>,
}

/// Represents the current state of a mouse button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonPosition {
    /// The mouse button is currently up.
    Up,
    /// The mouse button is currently down (pressed).
    Down(MouseButtonDown),
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
    /// Simple mouse event that is waiting to be consumed
    pub simple_event: Option<SimpleMouseEvent>,
    /// Map of MouseButton to ButtonState
    pub button_map: ButtonMap,
}


impl ButtonState {

    /// Constructor for a default ButtonState.
    pub fn new() -> ButtonState {
        ButtonState {
            was_just_released: false,
            was_just_pressed: false,
            position: ButtonPosition::Up,
            event: None
        }
    }

    /// Reset the `was_just_released` and `was_just_pressed` flags.
    pub fn reset_pressed_and_released(&mut self) {
        self.was_just_released = false;
        self.was_just_pressed = false;
    }

    /// returns true if the mouse button is currently Down, otherwise false
    pub fn is_down(&self) -> bool {
        match self.position {
            ButtonPosition::Down(_) => true,
            _ => false
        }
    }

    /// Returns true if the mouse button is currently Up, otherwise false
    pub fn is_up(&self) -> bool {
        match self.position {
            ButtonPosition::Up => true,
            _ => false
        }
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
            simple_event: None,
            button_map: ButtonMap::new(),
        }
    }

    /// Return the mouse state with its position relative to the given position.
    pub fn relative_to(self, xy: Point) -> Mouse {
        use ::vecmath::vec2_sub;

        let relative_simple_event  = self.simple_event.map(|mouse_event| mouse_event.relative_to(xy));
        Mouse { xy: vec2_sub(self.xy, xy),
            simple_event: relative_simple_event, ..self }
    }

    /// Call whenever the mouse moves, sets the new position of the mouse
    pub fn move_to(&mut self, xy: Point) {
        use input::MouseButton::{Left, Middle, Right};
        use self::ButtonPosition::Down;
        use self::simple_events::SimpleMouseEvent::Drag;

        let new_event: Option<SimpleMouseEvent> = {

            // These are the only buttons we care about at the moment.
            let buttons: Vec<(&ButtonState, MouseButton)> = vec![
                (&self.left, Left),
                (&self.middle, Middle),
                (&self.right, Right)
            ];

            buttons.iter()
                // Find the first button that is current in the Down position
                .filter(|state_and_button| state_and_button.0.is_down())
                .next().and_then(|state_and_button| {
                    // Once we have the button down info, map that to a MouseDragEvent
                    if let Down(button_down_info) = state_and_button.0.position {
                        Some(Drag(MouseDragEvent{
                            start: button_down_info,
                            current: MouseButtonDown{
                                time: SteadyTime::now(),
                                position: xy
                            },
                            mouse_button: state_and_button.1,
                            button_released: false
                        }))
                    } else {
                        None
                    }
                })
        };

        self.set_simple_event(new_event);

        // update the current position of the mouse
        self.xy = xy;
    }

    /// Sends the mouse a button down event
    pub fn button_down(&mut self, button: MouseButton) {
        use input::MouseButton::*;

        let mouse_position = self.xy.clone();

        let button_state = match button {
            Left => &mut self.left,
            Right => &mut self.right,
            Middle => &mut self.middle,
            _ => &mut self.unknown
        };
        button_state.position = ButtonPosition::Down(MouseButtonDown{
            time: SteadyTime::now(),
            position: mouse_position
        });
        button_state.was_just_pressed = true;
    }

    /// called when a mouse button is released
    pub fn button_up(&mut self, button: MouseButton) {
        use input::MouseButton::*;

        let current_mouse_position = self.xy.clone();
        let drag_distance_threshold = self.drag_distance_threshold;
        let mut new_simple_event: Option<SimpleMouseEvent> = None;
        {
            let button_state = match button {
                Left => &mut self.left,
                Right => &mut self.right,
                Middle => &mut self.middle,
                _ => &mut self.unknown
            };

            if let ButtonPosition::Down(mouse_down_info) = button_state.position {
                let drag_distance = distance_between(mouse_down_info.position, current_mouse_position);
                if drag_distance > drag_distance_threshold {
                    new_simple_event = Some(SimpleMouseEvent::Drag(MouseDragEvent{
                        mouse_button: button,
                        start: MouseButtonDown{ time: mouse_down_info.time, position: mouse_down_info.position },
                        current: MouseButtonDown{ time: SteadyTime::now(), position: current_mouse_position },
                        button_released: true
                    }));
                } else {
                    new_simple_event = Some(SimpleMouseEvent::Click(MouseClick{
                        mouse_button: button,
                        position: current_mouse_position
                    }));
                }
            }

            button_state.position = ButtonPosition::Up;
            button_state.was_just_released = true;
        }

        self.set_simple_event(new_simple_event);
    }

    /// called when the mouse wheel is scrolled
    pub fn scroll(&mut self, x: f64, y: f64) {
        self.set_simple_event(Some(SimpleMouseEvent::Scroll(Scroll{
            x: x,
            y: y
        })));
    }

    /// resets the state of the mouse
    pub fn reset(&mut self) {
        self.left.reset_pressed_and_released();
        self.right.reset_pressed_and_released();
        self.middle.reset_pressed_and_released();
        self.unknown.reset_pressed_and_released();
        self.simple_event = None;
        self.scroll.x = 0.0;
        self.scroll.y = 0.0;
    }

    /// Returns the current `SimpleMouseEvent` if there is one
    pub fn get_simple_event(&self) -> Option<SimpleMouseEvent> {
        self.simple_event
    }

    fn set_simple_event(&mut self, event: Option<SimpleMouseEvent>) {
        if event.is_some() {
            self.simple_event = event;
        }
    }

}

fn distance_between(a: Point, b: Point) -> Scalar {
    let dx_2 = (a[0] - b[0]).powi(2);
    let dy_2 = (a[1] - b[1]).powi(2);
    (dx_2 + dy_2).abs().sqrt()
}

#[test]
fn reset_should_reset_all_button_states_and_scroll_state() {
    let mut mouse = Mouse::new();
    mouse.left.was_just_pressed = true;
    mouse.left.was_just_released = true;
    mouse.right.was_just_pressed = true;
    mouse.right.was_just_released = true;
    mouse.middle.was_just_pressed = true;
    mouse.middle.was_just_released = true;
    mouse.unknown.was_just_pressed = true;
    mouse.unknown.was_just_released = true;
    mouse.scroll.x = 10.0;
    mouse.scroll.y = 20.0;
    mouse.simple_event = Some(SimpleMouseEvent::Scroll(Scroll{x: 2.0, y: 33.3}));

    mouse.reset();

    assert!(!mouse.left.was_just_pressed);
    assert!(!mouse.left.was_just_released);
    assert!(!mouse.right.was_just_pressed);
    assert!(!mouse.right.was_just_released);
    assert!(!mouse.middle.was_just_pressed);
    assert!(!mouse.middle.was_just_released);
    assert!(!mouse.unknown.was_just_pressed);
    assert!(!mouse.unknown.was_just_released);
    assert_eq!(0.0, mouse.scroll.x);
    assert_eq!(0.0, mouse.scroll.y);
    assert!(mouse.simple_event.is_none());
}

#[test]
fn scroll_should_create_a_simple_mouse_event_of_scroll() {
    let mut mouse = Mouse::new();
    mouse.scroll(2.0, 3.0);

    match mouse.get_simple_event() {
        Some(SimpleMouseEvent::Scroll(scroll_info)) => {
            assert_eq!(2.0, scroll_info.x);
            assert_eq!(3.0, scroll_info.y);
        },
        Some(thing) => panic!("Expected a SimpleMouseEvent::Scroll, instead got: {:?}", thing),
        _ => panic!("expected a SimpleMouseEvent::Scroll, instead got None")
    }
}

#[test]
fn distance_between_should_return_correct_distances_between_two_points() {
    let epsilon: Scalar = 0.0001;
    let test_cases: Vec<(Point, Point, Scalar)> = vec![
        /* (pointA, pointB, expected distance between) */
        ([0.0, 0.0], [10.0, 0.0], 10.0),
        ([-5.0, 0.0], [5.0, 0.0], 10.0),
        ([10.0, -5.0], [12.0, 5.0], 10.1980),
    ];

    for (point_a, point_b, expected_distance) in test_cases {
        let distance = distance_between(point_a, point_b);
        let abs_diff = (expected_distance - distance).abs();
        assert!(abs_diff <= epsilon,
            "expected distance between {:?} and {:?} to equal: {}, but was: {}",
            point_a, point_b, expected_distance, distance);
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
fn button_down_sets_button_state_to_down() {
    let mut mouse = Mouse::new();
    mouse.left.position = ButtonPosition::Up;

    mouse.button_down(MouseButton::Left);

    let is_down = match mouse.left.position {
        ButtonPosition::Down(_) => true,
        _ => false
    };
    assert!(is_down);
    assert!(mouse.left.was_just_pressed);
}

#[test]
fn button_up_sets_button_state_to_up() {
    let mut mouse = Mouse::new();
    mouse.left.position = ButtonPosition::Down(MouseButtonDown{
        time: SteadyTime::now(),
        position: [0.0, 0.0]
    });

    mouse.button_up(MouseButton::Left);
    assert_eq!(ButtonPosition::Up, mouse.left.position);
    assert!(mouse.left.was_just_released);
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
fn get_simple_event_returns_drag_event_if_mouse_was_dragged_without_releasing_button() {
    use input::MouseButton::Left;
    let mut mouse = Mouse::new();
    let start_position = mouse.xy;
    let new_position = [0.0, mouse.drag_distance_threshold + 1.0];
    mouse.button_down(Left);
    mouse.move_to(new_position);

    let actual_event = mouse.get_simple_event();
    assert!(actual_event.is_some());

    let maybe_drag_event: Option<MouseDragEvent> = match actual_event {
        Some(SimpleMouseEvent::Drag(mouse_drag_info)) => Some(mouse_drag_info),
        _ => None
    };
    assert!(maybe_drag_event.is_some());
    let drag_event = maybe_drag_event.unwrap();

    assert_eq!(start_position, drag_event.start.position);
    assert_eq!(new_position, drag_event.current.position);
    assert!(!drag_event.button_released);
}

#[test]
fn get_simple_event_returns_drag_event_if_mouse_was_dragged_then_button_released() {
    use input::MouseButton::Left;
    let mut mouse = Mouse::new();
    let start_position = mouse.xy;
    let new_position = [0.0, mouse.drag_distance_threshold + 1.0];
    mouse.button_down(Left);
    mouse.move_to(new_position);
    mouse.button_up(Left);

    let actual_event = mouse.get_simple_event();
    assert!(actual_event.is_some());

    let maybe_drag_event: Option<MouseDragEvent> = match actual_event {
        Some(SimpleMouseEvent::Drag(mouse_drag_info)) => Some(mouse_drag_info),
        _ => None
    };
    assert!(maybe_drag_event.is_some());
    let drag_event = maybe_drag_event.unwrap();

    assert_eq!(start_position, drag_event.start.position);
    assert_eq!(new_position, drag_event.current.position);
    assert!(drag_event.button_released);
}

#[test]
fn drag_event_has_original_start_position_after_multiple_mouse_move_events() {
    use input::MouseButton::Left;
    let mut mouse = Mouse::new();
    let start_position = mouse.xy;
    let final_position = [0.0, mouse.drag_distance_threshold + 1.0];
    mouse.button_down(Left);
    mouse.move_to([-5.0, 10.0]);
    mouse.move_to(final_position);

    let actual_event = mouse.get_simple_event();
    assert!(actual_event.is_some());

    let maybe_drag_event: Option<MouseDragEvent> = match actual_event {
        Some(SimpleMouseEvent::Drag(mouse_drag_info)) => Some(mouse_drag_info),
        _ => None
    };
    assert!(maybe_drag_event.is_some());
    let drag_event = maybe_drag_event.unwrap();

    assert_eq!(start_position, drag_event.start.position);
    assert_eq!(final_position, drag_event.current.position);
}

#[test]
fn get_simple_event_returns_click_event_if_mouse_was_dragged_less_than_drag_threshold() {
    use input::MouseButton::Left;
    let mut mouse = Mouse::new();
    let new_position = [0.0, mouse.drag_distance_threshold - 1.0];
    mouse.button_down(Left);
    mouse.move_to(new_position);
    mouse.button_up(Left);

    let actual_event = mouse.get_simple_event();
    assert!(actual_event.is_some());

    let maybe_click_event: Option<MouseClick> = match actual_event {
        Some(SimpleMouseEvent::Click(mouse_click)) => Some(mouse_click),
        _ => None
    };
    assert!(maybe_click_event.is_some());
    let click_event = maybe_click_event.unwrap();

    assert_eq!(new_position, click_event.position);
}
