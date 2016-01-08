//! module for abstracting over most mouse events.
//! Handles most common mouse events so that widgets don't have to
//! store any mouse state.

use input::MouseButton;
use position::Point;
use time::SteadyTime;
use ::vecmath::vec2_sub;

#[cfg(test)]
use position::Scalar;

// pub type MouseEventIterator = ();

/// Used for simplified mouse event handling. Most widgets can probably
/// just use these events
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MouseEvent {
    /// When the mouse button has been pressed but not yet released or dragged.
    /// This event is created as soon as the button is pressed.
    Down(ButtonDownEvent),
    /// Indicates that the mouse was clicked. A Click event is created when the mouse button is released, not depressed
    Click(MouseClick),
    /// Drag event is created when the mouse was moved over a certain threshold while a button was depressed
    Drag(MouseDragEvent),
    /// Scroll event is created when whenever the scroll wheel is moved
    Scroll(Scroll),
}


/// Info on a simple mouse click event. This event gets dispatched when a
/// mouse button goes down then up without moving more than the drag threshold
/// while the button is depressed.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseClick {
    /// Indicates Which button was clicked
    pub mouse_button: MouseButton,
    /// The Point describing the click location
    pub position: Point
}

/// Info on a simple mouse drag event. This event gets dispached when a mouse
/// button is depressed and the mouse is moved a distance greater than the
/// drag threshold. Holds the start and end positions of the drag, as well as
/// whether the mouse button is still being depressed.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseDragEvent {
    /// Which mouse button is being held during the drag
    pub mouse_button: MouseButton,
    /// The time and location where the drag was initiated (when the button was pressed)
    pub start: MouseButtonDown,
    /// The current time and location of the mouse
    pub current: MouseButtonDown,
    /// This will be false if the button is still being held down, or true if the button was released
    pub button_released: bool
}

/// Event that is created when a mouse button if first pressed (but not yet released)
/// a ButtonDownEvent will always precipitate either a Click or a Drag event.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ButtonDownEvent {
    /// The mouse button that was pressed
    pub mouse_button: MouseButton,
    /// The position of the mouse when the button was pressed
    pub position: Point,
}

/// Holds info on when a mouse button was depressed or released.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseButtonDown {
    /// The time that the mouse button was pressed.
    pub time: SteadyTime,
    /// The location of the mouse when the button was pressed
    pub position: Point
}

/// The amount of scrolling that has occurred since the last render event.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scroll {
    /// Scrolling across the x axis.
    pub x: f64,
    /// Scrolling across the y axis.
    pub y: f64,
}


impl MouseClick {

    /// Returns a new copy of the event data relative to the given point
    pub fn relative_to(&self, xy: Point) -> MouseClick {
        MouseClick{
            position: vec2_sub(self.position, xy),
            ..*self
        }
    }
}

impl MouseDragEvent {

    /// Returns a new copy of the event data relative to the given point
    pub fn relative_to(&self, xy: Point) -> MouseDragEvent {
        MouseDragEvent{
            start: self.start.relative_to(xy),
            current: self.current.relative_to(xy),
            ..*self
        }
    }
}

impl ButtonDownEvent {

    /// Returns a new copy of the event data relative to the given point
    pub fn relative_to(&self, xy: Point) -> ButtonDownEvent {
        ButtonDownEvent {
            position: vec2_sub(self.position, xy),
            ..*self
        }
    }
}

impl MouseEvent {

    /// Returns a new copy of the event data relative to the given point
    pub fn relative_to(&self, xy: Point) -> Self {
        use self::MouseEvent::*;

        match self {
            &Down(button_down) => Down(button_down.relative_to(xy)),
            &Click(mouse_click) => Click(mouse_click.relative_to(xy)),
            &Drag(mouse_drag) => Drag(mouse_drag.relative_to(xy)),
            &Scroll(scroll_info) => Scroll(scroll_info)
        }
    }
}

impl MouseButtonDown {

    /// Returns a new copy of the event data relative to the given point
    pub fn relative_to(&self, xy: Point) -> MouseButtonDown {
        MouseButtonDown{
            position: vec2_sub(self.position, xy),
            ..*self
        }
    }
}

#[test]
fn click_event_should_be_made_relative_to_a_point() {
    let click = MouseClick{
        mouse_button: MouseButton::Left,
        position: [10.0, 20.0]
    };

    let relative_click = click.relative_to([5.0, 10.0]);

    assert_float_eq(5.0, relative_click .position[0]);
    assert_float_eq(10.0, relative_click .position[1]);
}

#[test]
fn drag_event_should_be_made_relative_to_a_point() {
    let drag = MouseDragEvent{
        mouse_button: MouseButton::Left,
        start: MouseButtonDown{
            time: SteadyTime::now(),
            position: [4.0, -5.0]
        },
        current: MouseButtonDown{
            time: SteadyTime::now(),
            position: [24.0, -10.0]
        },
        button_released: false
    };

    let relative_drag = drag.relative_to([20.0, -5.0]);
    assert_float_eq(-16.0, relative_drag.start.position[0]);
    assert_float_eq(0.0, relative_drag.start.position[1]);
    assert_float_eq(4.0, relative_drag.current.position[0]);
    assert_float_eq(-5.0, relative_drag.current.position[1]);
}

#[cfg(test)]
pub fn assert_float_eq(a: Scalar, b: Scalar) {
    let epsilon = 0.0001;
    assert!((a - epsilon) <= b && (a + epsilon) >= b, format!("{} != {}", a, b));
}
