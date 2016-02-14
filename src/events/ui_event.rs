//! Contains all the structs and enums to describe all of the input events that `Widget`s
//! can handle. The core of this module is the `UiEvent` enum, which encapsulates all
//! of those events.

use input::{Input, MouseButton, Motion, Button};
use input::keyboard::ModifierKey;
use position::Point;
use vecmath::vec2_sub;
use widget::Index;

/// Enum containing all the events that `Widget`s can listen for.
#[derive(Clone, PartialEq, Debug)]
pub enum UiEvent {
    /// Represents a raw `input::Input` event
    Raw(Input),
    /// Represents a mouse button being pressed and subsequently released while the
    /// mouse stayed in roughly the same place.
    MouseClick(MouseClick),
    /// Represents a mouse button being pressed and a subsequent movement of the mouse.
    MouseDrag(MouseDrag),
    /// This is a generic scroll event. This is different from the `input::Movement::MouseScroll`
    /// event in several aspects. For one, it does not necessarily have to get created by a
    /// mouse wheel, it could be generated from a keypress, or as a response to handling some
    /// other event. Secondly, it contains a field holding the `input::keyboard::ModifierKey`
    /// that was held while the scroll occured.
    Scroll(Scroll),
    /// Indicates that the given widget is starting to capture the mouse.
    WidgetCapturesMouse(Index),
    /// Indicates that the given widget is losing mouse capture.
    WidgetUncapturesMouse(Index),
    /// Indicates that the given widget is starting to capture the keyboard.
    WidgetCapturesKeyboard(Index),
    /// Indicates that the given widget is losing keyboard capture.
    WidgetUncapturesKeyboard(Index),
}

/// Contains all the relevant information for a mouse drag.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MouseDrag {
    /// Which mouse button was being held during the drag
    pub button: MouseButton,
    /// The origin of the drag. This will always be the position of the mouse whenever the
    /// button was first pressed
    pub start: Point,
    /// The end position of the mouse. If `in_progress` is true, then subsequent `MouseDrag`
    /// events may be created with a new `end` as the mouse continues to move.
    pub end: Point,
    /// Which modifier keys are being held during the mouse drag.
    pub modifier: ModifierKey,
    /// Indicates whether the mouse button is still being held down. If it is, then
    /// `in_progress` will be `true` and more `MouseDrag` events can likely be expected.
    pub in_progress: bool,
}

/// Contains all the relevant information for a mouse click.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MouseClick {
    /// Which mouse button was clicked
    pub button: MouseButton,
    /// The location of the click
    pub location: Point,
    /// Which modifier keys, if any, that were being held down when the user clicked
    pub modifier: ModifierKey,
}

/// Holds all the relevant information about a scroll event
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Scroll {
    /// The amount of scroll along the x axis.
    pub x: f64,
    /// The amount of scroll along the y axis.
    pub y: f64,
    /// Which modifier keys, if any, that were being held down while the scroll occured
    pub modifiers: ModifierKey,
}

/// Marks something that can be made relative to another `Point`.
pub trait RelativePosition {
    /// Returns a copy of `Self` that is relative to the given `Point`.
    /// All `Point`s are assumed to be relative to center (0,0).
    fn relative_to(&self, xy: Point) -> Self;
}

impl RelativePosition for MouseClick {
    fn relative_to(&self, xy: Point) -> MouseClick {
        MouseClick{
            location: vec2_sub(self.location, xy),
            ..*self
        }
    }
}

impl RelativePosition for MouseDrag {
    fn relative_to(&self, xy: Point) -> MouseDrag {
        MouseDrag{
            start: vec2_sub(self.start, xy),
            end: vec2_sub(self.end, xy),
            ..*self
        }
    }
}

impl RelativePosition for UiEvent {
    fn relative_to(&self, xy: Point) -> Self {
        use self::UiEvent::{MouseClick, MouseDrag, Raw};
        match *self {
            MouseClick(click) => MouseClick(click.relative_to(xy)),
            MouseDrag(drag) => MouseDrag(drag.relative_to(xy)),
            Raw(ref raw_input) => Raw(raw_input.relative_to(xy)),
            ref other_event => other_event.clone()
        }
    }
}

impl RelativePosition for Input {
    fn relative_to(&self, xy: Point) -> Input {
        match *self {
            Input::Move(Motion::MouseRelative(x, y)) =>
                Input::Move(Motion::MouseRelative(x - xy[0], y - xy[1])),
            Input::Move(Motion::MouseCursor(x, y)) =>
                Input::Move(Motion::MouseCursor(x - xy[0], y - xy[1])),
            ref other_input => other_input.clone()
        }
    }
}

impl UiEvent {
    /// Returns `true` if this event is related to the mouse. Note that just because this method
    /// returns true does not mean that the event necessarily came from the mouse.
    /// A `UiEvent::Scroll` is considered to be both a mouse and a keyboard event.
    pub fn is_mouse_event(&self) -> bool {
        match *self {
            UiEvent::Raw(Input::Press(Button::Mouse(_))) => true,
            UiEvent::Raw(Input::Release(Button::Mouse(_))) => true,
            UiEvent::Raw(Input::Move(Motion::MouseCursor(_, _))) => true,
            UiEvent::Raw(Input::Move(Motion::MouseRelative(_, _))) => true,
            UiEvent::Raw(Input::Move(Motion::MouseScroll(_, _))) => true,
            UiEvent::MouseClick(_) => true,
            UiEvent::MouseDrag(_) => true,
            UiEvent::Scroll(_) => true,
            _ => false
        }
    }

    /// Returns `true` if this event is related to the keyboard. Note that just because this method
    /// returns true does not mean that the event necessarily came from the keyboard.
    /// A `UiEvent::Scroll` is considered to be both a mouse and a keyboard event.
    pub fn is_keyboard_event(&self) -> bool {
        match *self {
            UiEvent::Raw(Input::Press(Button::Keyboard(_))) => true,
            UiEvent::Raw(Input::Release(Button::Keyboard(_))) => true,
            UiEvent::Raw(Input::Text(_)) => true,
            UiEvent::Scroll(_) => true,
            _ => false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use input::{Input, MouseButton, Motion, Button, JoystickAxisArgs};
    use input::keyboard::{self, Key, NO_MODIFIER};

    // We'll see if this approach causes problems later on down the road...
    #[test]
    fn scroll_event_shoulbe_be_both_a_mouse_and_keyboard_event() {
        let scroll_event = UiEvent::Scroll(Scroll{
            x: 0.0,
            y: 0.0,
            modifiers: NO_MODIFIER
        });
        assert!(scroll_event.is_mouse_event());
        assert!(scroll_event.is_keyboard_event());
    }

    #[test]
    fn is_keyboard_event_should_be_true_for_all_keyboard_events() {
        let keyboard_events = vec![
            UiEvent::Raw(Input::Press(Button::Keyboard(Key::L))),
            UiEvent::Raw(Input::Release(Button::Keyboard(Key::L))),
            UiEvent::Raw(Input::Text("wha?".to_string())),
        ];
        for event in keyboard_events {
            assert!(event.is_keyboard_event(), format!("{:?} should be a keyboard event", event));
        }

        let non_keyboard_events = vec![
            UiEvent::Raw(Input::Press(Button::Mouse(MouseButton::Left))),
            UiEvent::Raw(Input::Release(Button::Mouse(MouseButton::Left))),
            UiEvent::MouseClick(MouseClick{
                button: MouseButton::Left,
                location: [0.0, 0.0],
                modifier: NO_MODIFIER
            }),
            UiEvent::MouseDrag(MouseDrag{
                button: MouseButton::Left,
                start: [0.0, 0.0],
                end: [0.0, 0.0],
                modifier: NO_MODIFIER,
                in_progress: true,
            }),
            UiEvent::Raw(Input::Move(Motion::MouseCursor(2.0, 3.0))),
            UiEvent::Raw(Input::Move(Motion::MouseRelative(2.0, 3.0))),
            UiEvent::Raw(Input::Move(Motion::MouseScroll(3.5, 6.5))),
        ];

        for event in non_keyboard_events {
            assert!(!event.is_keyboard_event(), format!("{:?} should not be a keyboard event", event));
        }
    }

    #[test]
    fn is_mouse_event_should_be_true_for_all_mouse_events() {
        let mouse_events = vec![
            UiEvent::Raw(Input::Press(Button::Mouse(MouseButton::Left))),
            UiEvent::Raw(Input::Release(Button::Mouse(MouseButton::Left))),
            UiEvent::MouseClick(MouseClick{
                button: MouseButton::Left,
                location: [0.0, 0.0],
                modifier: NO_MODIFIER
            }),
            UiEvent::MouseDrag(MouseDrag{
                button: MouseButton::Left,
                start: [0.0, 0.0],
                end: [0.0, 0.0],
                modifier: NO_MODIFIER,
                in_progress: true,
            }),
            UiEvent::Raw(Input::Move(Motion::MouseCursor(2.0, 3.0))),
            UiEvent::Raw(Input::Move(Motion::MouseRelative(2.0, 3.0))),
            UiEvent::Raw(Input::Move(Motion::MouseScroll(3.5, 6.5))),
        ];
        for event in mouse_events {
            assert!(event.is_mouse_event(), format!("{:?}.is_mouse_event() == false", event));
        }

        let non_mouse_events = vec![
            UiEvent::Raw(Input::Press(Button::Keyboard(Key::G))),
            UiEvent::Raw(Input::Release(Button::Keyboard(Key::G))),
            UiEvent::Raw(Input::Move(Motion::JoystickAxis(JoystickAxisArgs{
                id: 0,
                axis: 0,
                position: 0f64
            }))),
            UiEvent::Raw(Input::Text("rust is brown".to_string())),
            UiEvent::Raw(Input::Resize(0, 0)),
            UiEvent::Raw(Input::Focus(true)),
            UiEvent::Raw(Input::Cursor(true)),
        ];
        for event in non_mouse_events {
            assert!(!event.is_mouse_event(), format!("{:?}.is_mouse_event() == true", event));
        }
    }

    #[test]
    fn mouse_click_should_be_made_relative() {
        let original = UiEvent::MouseClick(MouseClick{
            button: MouseButton::Middle,
            location: [30.0, -80.0],
            modifier: keyboard::SHIFT
        });
        let relative = original.relative_to([10.0, 20.0]);

        if let UiEvent::MouseClick(click) = relative {
            assert_eq!([20.0, -100.0], click.location);
            assert_eq!(MouseButton::Middle, click.button);
            assert_eq!(keyboard::SHIFT, click.modifier);
        } else {
            panic!("expected a mouse click");
        }
    }

    #[test]
    fn mouse_drage_should_be_made_relative() {
        let original = UiEvent::MouseDrag(MouseDrag{
            start: [20.0, 5.0],
            end: [50.0, 1.0],
            button: MouseButton::Left,
            modifier: keyboard::CTRL,
            in_progress: false
        });

        let relative = original.relative_to([-5.0, 5.0]);
        if let UiEvent::MouseDrag(drag) = relative {
            assert_eq!([25.0, 0.0], drag.start);
            assert_eq!([55.0, -4.0], drag.end);
            assert_eq!(MouseButton::Left, drag.button);
            assert_eq!(keyboard::CTRL, drag.modifier);
            assert!(!drag.in_progress);
        } else {
            panic!("expected to get a drag event");
        }
    }

    #[test]
    fn mouse_cursor_should_be_made_relative() {
        let original = UiEvent::Raw(Input::Move(Motion::MouseCursor(-44.0, 55.0)));
        let relative = original.relative_to([4.0, 5.0]);
        if let UiEvent::Raw(Input::Move(Motion::MouseCursor(x, y))) = relative {
            assert_eq!(-48.0, x);
            assert_eq!(50.0, y);
        } else {
            panic!("expected a mouse move event");
        }
    }

    #[test]
    fn mouse_relative_motion_should_be_made_relative() {
        let original = UiEvent::Raw(Input::Move(Motion::MouseRelative(-2.0, -4.0)));
        let relative = original.relative_to([3.0, 3.0]);
        if let UiEvent::Raw(Input::Move(Motion::MouseRelative(x, y))) = relative {
            assert_eq!(-5.0, x);
            assert_eq!(-7.0, y);
        } else {
            panic!("expected a mouse relative motion event");
        }
    }
}
