//!
//! A module for describing Mouse state.
//!
//! The `Ui` will continuously maintain the latest Mouse state, necessary for widget logic.
//!

pub use input::MouseButton;
pub use graphics::math::Scalar;
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
}

/// Represents the current state of a mouse button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonPosition {
    /// The mouse button is currently up.
    Up,
    /// The mouse button is currently down (pressed).
    Down(SteadyTime, Point),
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
}


/// Used for simplified mouse event handling. Most widgets can probably
/// just use these events
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SimpleMouseEvent {
    Click(MouseClick),
    Drag(MouseDragEvent),
}

impl SimpleMouseEvent {

    pub fn relative_to(&self, xy: Point) -> Self {
        use self::SimpleMouseEvent::*;

        match self {
            &Click(mouse_click) => Click(mouse_click.relative_to(xy)),
            &Drag(mouse_drag) => Drag(mouse_drag.relative_to(xy))
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseClick {
    mouse_button: MouseButton,
    position: Point
}

impl MouseClick {

    pub fn relative_to(&self, xy: Point) -> MouseClick {
        use ::vecmath::vec2_sub;

        MouseClick{
            position: vec2_sub(self.position, xy),
            ..*self
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseDragEvent {
    mouse_button: MouseButton,
    start: MouseButtonDown,
    current: MouseButtonDown,
}

impl MouseDragEvent {

    pub fn relative_to(&self, xy: Point) -> MouseDragEvent {
        MouseDragEvent{
            start: self.start.relative_to(xy),
            current: self.current.relative_to(xy),
            ..*self
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct MouseButtonDown {
    time: SteadyTime,
    position: Point
}

impl MouseButtonDown {

    pub fn relative_to(&self, xy: Point) -> MouseButtonDown {
        use ::vecmath::vec2_sub;

        MouseButtonDown{
            position: vec2_sub(self.position, xy),
            ..*self
        }
    }
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

    /// returns true if the mouse button is currently Down, otherwise false
    pub fn is_down(&self) -> bool {
        match self.position {
            ButtonPosition::Down(_, _) => true,
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
            simple_event: None
        }
    }

    /// Return the mouse state with its position relative to the given position.
    pub fn relative_to(self, xy: Point) -> Mouse {
        use ::vecmath::vec2_sub;

        let relative_simple_event  = self.simple_event.map(|mouse_event| mouse_event.relative_to(xy));
        Mouse { xy: vec2_sub(self.xy, xy),
            simple_event: relative_simple_event, ..self }
    }

    /// sets the new position of the mouse
    pub fn move_to(&mut self, xy: Point) {
        use self::ButtonPosition::Down;

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
        button_state.position = ButtonPosition::Down(SteadyTime::now(), mouse_position);
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

            if let ButtonPosition::Down(time, start_position) = button_state.position {
                let drag_distance = distance_between(start_position, current_mouse_position);
                if drag_distance > drag_distance_threshold {
                    new_simple_event = Some(SimpleMouseEvent::Drag(MouseDragEvent{
                        mouse_button: button,
                        start: MouseButtonDown{ time: time, position: start_position },
                        current: MouseButtonDown{ time: SteadyTime::now(), position: current_mouse_position }
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

        self.simple_event = new_simple_event;
    }

    /// indicates whether or not the mouse was clicked
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

    /// Returns the current `SimpleMouseEvent` if there is one
    pub fn get_simple_event(&self) -> Option<SimpleMouseEvent> {
        self.simple_event
    }

}

fn distance_between(a: Point, b: Point) -> Scalar {
    let dx_2 = (a[0] - b[0]).powi(2);
    let dy_2 = (a[1] - b[1]).powi(2);
    (dx_2 + dy_2).abs().sqrt()
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
        ButtonPosition::Down(_, _) => true,
        _ => false
    };
    assert!(is_down);
}

#[test]
fn button_up_sets_button_state_to_up() {
    let mut mouse = Mouse::new();
    mouse.left.position = ButtonPosition::Down(SteadyTime::now(), [0.0, 0.0]);

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
}

#[test]
fn get_simple_event_returns_click_event_if_mouse_was_dragged_less_than_drag_threshold() {
    use input::MouseButton::Left;
    let mut mouse = Mouse::new();
    let start_position = mouse.xy;
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
