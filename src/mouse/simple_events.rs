
pub use input::MouseButton;

use position::Point;
use time::SteadyTime;

/// Used for simplified mouse event handling. Most widgets can probably
/// just use these events
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SimpleMouseEvent {
    /// Indicates that the mouse was clicked. A Click event is created when the mouse button is released, not depressed
    Click(MouseClick),
    /// Drag event is created when the mouse was moved over a certain threshold while a button was depressed
    Drag(MouseDragEvent),
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseClick {
    /// Indicates Which button was clicked
    pub mouse_button: MouseButton,
    /// The Point describing the click location
    pub position: Point
}

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MouseButtonDown {
    /// The time that the mouse button was pressed.
    pub time: SteadyTime,
    /// The location of the mouse when the button was pressed
    pub position: Point
}

impl MouseClick {

    /// Returns a new copy of the event data relative to the given point
    pub fn relative_to(&self, xy: Point) -> MouseClick {
        use ::vecmath::vec2_sub;

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

impl SimpleMouseEvent {

    /// Returns a new copy of the event data relative to the given point
    pub fn relative_to(&self, xy: Point) -> Self {
        use self::SimpleMouseEvent::*;

        match self {
            &Click(mouse_click) => Click(mouse_click.relative_to(xy)),
            &Drag(mouse_drag) => Drag(mouse_drag.relative_to(xy))
        }
    }
}

impl MouseButtonDown {

    /// Returns a new copy of the event data relative to the given point
    pub fn relative_to(&self, xy: Point) -> MouseButtonDown {
        use ::vecmath::vec2_sub;

        MouseButtonDown{
            position: vec2_sub(self.position, xy),
            ..*self
        }
    }
}
