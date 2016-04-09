//! Contains all the structs and enums to describe all of the input events that `Widget`s
//! can handle.
//!
//! The core of this module is the `UiEvent` enum, which encapsulates all of those events.

use input::{keyboard, Button, MouseButton};
use position::Point;
use vecmath::vec2_sub;
use widget;

#[doc(inline)]
pub use backend::event::{Input, Motion};

/// Enum containing all the events that `Widget`s can listen for.
#[derive(Clone, PartialEq, Debug)]
pub enum UiEvent {
    /// Represents a raw `input::Input` event
    Raw(Input),
    /// Represents a pointing device being pressed and subsequently released while over the same
    /// location.
    Click(Click),
    /// Represents a pointing device button being pressed and a subsequent movement of the mouse.
    Drag(Drag),
    /// This is a generic scroll event. This is different from the `input::Movement::MouseScroll`
    /// event in several aspects.
    ///
    /// For one, it does not necessarily have to get created by a mouse wheel, it could be
    /// generated from a keypress, or as a response to handling some other event.
    ///
    /// Secondly, it contains a field holding the `input::keyboard::ModifierKey` that was held
    /// while the scroll occured.
    Scroll(Scroll),
    /// Indicates that the given widget is starting to capture the mouse.
    WidgetCapturesMouse(widget::Index),
    /// Indicates that the given widget is losing mouse capture.
    WidgetUncapturesMouse(widget::Index),
    /// Indicates that the given widget is starting to capture the keyboard.
    WidgetCapturesKeyboard(widget::Index),
    /// Indicates that the given widget is losing keyboard capture.
    WidgetUncapturesKeyboard(widget::Index),
}

/// Contains all the relevant information for a mouse drag.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Drag {
    /// Which mouse button was being held during the drag
    pub button: MouseButton,
    /// The origin of the drag. This will always be the position of the mouse whenever the
    /// button was first pressed
    pub start: Point,
    /// The end position of the mouse.
    pub end: Point,
    /// Which modifier keys are being held during the mouse drag.
    pub modifiers: keyboard::ModifierKey,
    /// The widget that was under the mouse when `button` was first pressed.
    pub widget: Option<widget::Index>,
}

/// Contains all the relevant information for a mouse click.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Click {
    /// Which mouse button was clicked
    pub button: MouseButton,
    /// The position at which the mouse was released.
    pub xy: Point,
    /// Which modifier keys, if any, that were being held down when the user clicked
    pub modifiers: keyboard::ModifierKey,
    /// The widget that was clicked if any.
    ///
    /// Note that this is only be `Some` if the widget was under the mouse during both the press
    /// *and* release of `button`.
    pub widget: Option<widget::Index>,
}

/// Holds all the relevant information about a scroll event
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Scroll {
    /// The amount of scroll along the x axis.
    pub x: f64,
    /// The amount of scroll along the y axis.
    pub y: f64,
    /// Which modifier keys, if any, that were being held down while the scroll occured
    pub modifiers: keyboard::ModifierKey,
}

impl Click {
    /// Returns a copy of the Click relative to the given `position::Point`
    pub fn relative_to(&self, xy: Point) -> Click {
        Click {
            xy: vec2_sub(self.xy, xy),
            ..*self
        }
    }
}

impl Drag {
    /// Returns a copy of the Drag relative to the given `position::Point`
    pub fn relative_to(&self, xy: Point) -> Drag {
        Drag{
            start: vec2_sub(self.start, xy),
            end: vec2_sub(self.end, xy),
            ..*self
        }
    }
}

impl UiEvent {

    /// Returns a copy of the UiEvent relative to the given `position::Point`
    pub fn relative_to(self, xy: Point) -> Self {
        use self::UiEvent::{Click, Drag, Raw};
        match self {
            Click(click) => Click(click.relative_to(xy)),
            Drag(drag) => Drag(drag.relative_to(xy)),
            Raw(Input::Move(Motion::MouseRelative(x, y))) =>
                Raw(Input::Move(Motion::MouseRelative(x - xy[0], y - xy[1]))),
            Raw(Input::Move(Motion::MouseCursor(x, y))) =>
                Raw(Input::Move(Motion::MouseCursor(x - xy[0], y - xy[1]))),
            other_event => other_event
        }
    }

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
            UiEvent::Click(_) => true,
            UiEvent::Drag(_) => true,
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

impl From<Input> for UiEvent {
    fn from(input: Input) -> Self {
        UiEvent::Raw(input)
    }
}

#[cfg(test)]
mod test {
    use event::{self, Input, Motion, UiEvent};
    use input::{Button, MouseButton};
    use input::keyboard::{self, Key, NO_MODIFIER};

    // We'll see if this approach causes problems later on down the road...
    #[test]
    fn scroll_event_shoulbe_be_both_a_mouse_and_keyboard_event() {
        let scroll_event = UiEvent::Scroll(event::Scroll{
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
            UiEvent::Click(event::Click {
                button: MouseButton::Left,
                xy: [0.0, 0.0],
                modifiers: NO_MODIFIER,
                widget: None,
            }),
            UiEvent::Drag(event::Drag {
                button: MouseButton::Left,
                start: [0.0, 0.0],
                end: [0.0, 0.0],
                modifiers: NO_MODIFIER,
                widget: None,
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
            UiEvent::Click(event::Click {
                button: MouseButton::Left,
                xy: [0.0, 0.0],
                modifiers: NO_MODIFIER,
                widget: None,
            }),
            UiEvent::Drag(event::Drag {
                button: MouseButton::Left,
                start: [0.0, 0.0],
                end: [0.0, 0.0],
                modifiers: NO_MODIFIER,
                widget: None,
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
        let original = UiEvent::Click(event::Click {
            button: MouseButton::Middle,
            xy: [30.0, -80.0],
            modifiers: keyboard::SHIFT,
            widget: None,
        });
        let relative = original.relative_to([10.0, 20.0]);

        if let UiEvent::Click(click) = relative {
            assert_eq!([20.0, -100.0], click.xy);
            assert_eq!(MouseButton::Middle, click.button);
            assert_eq!(keyboard::SHIFT, click.modifiers);
        } else {
            panic!("expected a mouse click");
        }
    }

    #[test]
    fn mouse_drage_should_be_made_relative() {
        let original = UiEvent::Drag(event::Drag {
            start: [20.0, 5.0],
            end: [50.0, 1.0],
            button: MouseButton::Left,
            modifiers: keyboard::CTRL,
            widget: None,
        });

        let relative = original.relative_to([-5.0, 5.0]);
        if let UiEvent::Drag(drag) = relative {
            assert_eq!([25.0, 0.0], drag.start);
            assert_eq!([55.0, -4.0], drag.end);
            assert_eq!(MouseButton::Left, drag.button);
            assert_eq!(keyboard::CTRL, drag.modifiers);
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
