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
    /// Amount that the mouse has scrolled since the last render.
    pub scroll: Scroll,
    /// Movements less than this threshold will not be considered drags
    pub drag_distance_threshold: Scalar,
    /// Map of MouseButton to ButtonState
    pub buttons: ButtonMap,
    /// Holds a scroll event if there is one
    pub maybe_scroll_event: Option<SimpleMouseEvent>,
}

/// Iterator over mouse events.
pub struct MouseEventIterator<'a>{
    mouse: &'a Mouse,
    idx: u32,
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
            scroll: Scroll { x: 0.0, y: 0.0 },
            drag_distance_threshold: 2.0,
            maybe_scroll_event: None,
            buttons: ButtonMap::new(),
        }
    }

    /// Return the mouse state with all `position::Point`s relative to the given point.
    pub fn relative_to(self, xy: Point) -> Mouse {
        use ::vecmath::vec2_sub;

        let relative_button_map = self.buttons.relative_to(xy);
        Mouse { xy: vec2_sub(self.xy, xy),
            buttons: relative_button_map, ..self }
    }

    /// Call whenever the mouse moves, sets the new position of the mouse and
    /// creates a Drag event if a button is being held down.
    pub fn move_to(&mut self, xy: Point) {
        // Check to see if moving the mouse will create a Drag event.
        // Update any events if it does.
        let buttons_and_events = self.buttons.all_buttons().iter()
            // Find the first button that is current in the Down position
            .filter(|button_and_state| button_and_state.1.is_down())
            .map(|button_and_state| {
                // Once we have the button down info, map that to a MouseDragEvent
                let maybe_event = self.get_drag_event_for_move(button_and_state.0, button_and_state.1, xy);
                (button_and_state.0, maybe_event)
            }).filter(|button_and_event| button_and_event.1.is_some())
            .collect::<Vec<(MouseButton, Option<SimpleMouseEvent>)>>();

        for button_and_event in buttons_and_events {
            self.buttons.get_mut(button_and_event.0).event = button_and_event.1;
        }

        // update the current position of the mouse
        self.xy = xy;
    }

    /// Sends the mouse a button down event
    pub fn button_down(&mut self, button: MouseButton) {
        let mouse_position = self.xy.clone();

        self.buttons.update(button, |state| {
            state.position = ButtonPosition::Down(MouseButtonDown{
                time: SteadyTime::now(),
                position: mouse_position
            });
            state.was_just_pressed = true;
        });
    }

    /// called when a mouse button is released
    pub fn button_up(&mut self, button: MouseButton) {
        use self::ButtonPosition::Down;

        let new_simple_event = match self.buttons.get(button).position {
            ButtonPosition::Down(mouse_down_info) if self.is_drag(mouse_down_info.position, self.xy) => {
                Some(SimpleMouseEvent::Drag(MouseDragEvent{
                    mouse_button: button,
                    start: MouseButtonDown{ time: mouse_down_info.time, position: mouse_down_info.position },
                    current: MouseButtonDown{ time: SteadyTime::now(), position: self.xy },
                    button_released: true
                }))
            },
            ButtonPosition::Down(_) => {
                Some(SimpleMouseEvent::Click(MouseClick{
                    mouse_button: button,
                    position: self.xy
                }))
            },
            _ => None
        };

        self.buttons.update(button, |state| {
            state.position = ButtonPosition::Up;
            state.was_just_released = true;
            state.event = new_simple_event;
        });
    }

    /// called when the mouse wheel is scrolled
    pub fn scroll(&mut self, x: f64, y: f64) {
        self.maybe_scroll_event = Some(SimpleMouseEvent::Scroll(Scroll{
            x: x,
            y: y
        }));
    }

    /// resets the state of the mouse
    pub fn reset(&mut self) {
        for button_state in self.buttons.iter_mut() {
            button_state.reset_pressed_and_released();
            button_state.event = None;
        }
        self.maybe_scroll_event = None;
        self.scroll.x = 0.0;
        self.scroll.y = 0.0;
    }

    /// Returns any mouse events since the last update.
    pub fn events(&self) -> MouseEventIterator {
        MouseEventIterator::new(self)
    }

    fn is_drag(&self, start_position: Point, end_position: Point) -> bool {
        distance_between(start_position, end_position) >= self.drag_distance_threshold
    }

    fn get_drag_event_for_move(&self, button: MouseButton,
                                current_button_state: &ButtonState,
                                current_xy: Point) -> Option<SimpleMouseEvent> {
        use self::ButtonPosition::Down;
        use self::simple_events::SimpleMouseEvent::Drag;

        match current_button_state.position {
            Down(info) if self.is_drag(info.position, current_xy) => {
                Some(Drag(MouseDragEvent{
                    start: info,
                    current: MouseButtonDown{
                        time: SteadyTime::now(),
                        position: current_xy
                    },
                    mouse_button: button,
                    button_released: false
                }))
            },
            _ => None
        }
    }

}

fn distance_between(a: Point, b: Point) -> Scalar {
    let dx_2 = (a[0] - b[0]).powi(2);
    let dy_2 = (a[1] - b[1]).powi(2);
    (dx_2 + dy_2).abs().sqrt()
}

impl<'a> ::std::iter::Iterator for MouseEventIterator<'a> {
    type Item = SimpleMouseEvent;

    fn next(&mut self) -> Option<SimpleMouseEvent> {
        let mut evt: Option<SimpleMouseEvent> = None;
        while (self.idx as usize) < NUM_MOUSE_BUTTONS && evt.is_none() {
            let button_state = self.mouse.buttons.get(MouseButton::from(self.idx));
            evt = button_state.event;
            self.idx += 1;
        }
        if (self.idx as usize) == NUM_MOUSE_BUTTONS && evt.is_none() {
            evt = self.mouse.maybe_scroll_event;
        }
        evt
    }
}

impl<'a> MouseEventIterator<'a> {
    fn new(mouse: &Mouse) -> MouseEventIterator {
        MouseEventIterator{
            mouse: mouse,
            idx: 0
        }
    }
}

#[test]
fn reset_should_reset_all_button_states_and_scroll_state() {
    let mut mouse = Mouse::new();
    for button in mouse.buttons.iter_mut() {
        button.was_just_pressed = true;
        button.was_just_released = true;
    }
    mouse.scroll.x = 10.0;
    mouse.scroll.y = 20.0;
    mouse.maybe_scroll_event = Some(SimpleMouseEvent::Scroll(Scroll{x: 2.0, y: 33.3}));

    mouse.reset();

    for button in mouse.buttons.iter() {
        assert!(!button.was_just_pressed);
        assert!(!button.was_just_released);
    }
    assert_eq!(0.0, mouse.scroll.x);
    assert_eq!(0.0, mouse.scroll.y);
    assert!(mouse.maybe_scroll_event.is_none());
    assert_eq!(0, mouse.events().count());
}

#[test]
fn scroll_should_create_a_new_scroll_event() {
    let mut mouse = Mouse::new();
    mouse.scroll(2.0, 3.0);

    match mouse.maybe_scroll_event {
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
    mouse.buttons.get_mut(MouseButton::Left).position = ButtonPosition::Up;

    mouse.button_down(MouseButton::Left);
    let button_state = mouse.buttons.get(MouseButton::Left);
    assert!(button_state.is_down());
    assert!(button_state.was_just_pressed);
}

#[test]
fn button_up_sets_button_state_to_up() {
    let mut mouse = Mouse::new();
    mouse.buttons.update(MouseButton::Left, |state| {
        state.position = ButtonPosition::Down(MouseButtonDown{
            time: SteadyTime::now(),
            position: [0.0, 0.0]
        });
    });

    mouse.button_up(MouseButton::Left);

    let state = mouse.buttons.get(MouseButton::Left);
    assert_eq!(ButtonPosition::Up, state.position);
    assert!(state.was_just_released);
}

#[test]
fn events_returns_click_if_button_goes_down_then_up_in_same_position() {
    use input::MouseButton::Right;
    let mut mouse = Mouse::new();
    mouse.button_down(Right);
    mouse.button_up(Right);
    let expected_event = SimpleMouseEvent::Click(MouseClick{
        mouse_button: Right,
        position: mouse.xy.clone()
    });

    let actual_event = mouse.events().next();
    assert!(actual_event.is_some());
    assert_eq!(expected_event, actual_event.unwrap());
}

#[test]
fn events_returns_drag_event_if_mouse_was_dragged_without_releasing_button() {
    use input::MouseButton::Left;
    let mut mouse = Mouse::new();
    let start_position = mouse.xy;
    let new_position = [0.0, mouse.drag_distance_threshold + 1.0];
    mouse.button_down(Left);
    mouse.move_to(new_position);

    let actual_event = mouse.events().next();
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
fn drag_event_is_not_created_when_mouse_is_dragged_less_than_threshold() {
    use input::MouseButton::Left;
    let mut mouse = Mouse::new();
    let new_position = [0.0, mouse.drag_distance_threshold - 1.0];
    mouse.button_down(Left);
    mouse.move_to(new_position);
    // mouse button stays down

    let actual_event = mouse.events().next();
    assert!(actual_event.is_none());
}

#[test]
fn events_returns_drag_event_if_mouse_was_dragged_then_button_released() {
    use input::MouseButton::Left;
    let mut mouse = Mouse::new();
    let start_position = mouse.xy;
    let new_position = [0.0, mouse.drag_distance_threshold + 1.0];
    mouse.button_down(Left);
    mouse.move_to(new_position);
    mouse.button_up(Left);

    let actual_event = mouse.events().next();
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

    let actual_event = mouse.events().next();
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

    let actual_event = mouse.events().next();
    assert!(actual_event.is_some());

    let maybe_click_event: Option<MouseClick> = match actual_event {
        Some(SimpleMouseEvent::Click(mouse_click)) => Some(mouse_click),
        _ => None
    };
    assert!(maybe_click_event.is_some());
    let click_event = maybe_click_event.unwrap();

    assert_eq!(new_position, click_event.position);
}
