use input::{Input, MouseButton, Motion, Button};
use input::keyboard::ModifierKey;
use position::Point;
use vecmath::vec2_sub;
use widget::Index;

#[derive(Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum ConrodEvent {
    Raw(Input),
    MouseClick(MouseClick),
    MouseDrag(MouseDrag),
    Scroll(Scroll),
    WidgetCapturesMouse(Index),
    WidgetUncapturesMouse(Index),
    WidgetCapturesKeyboard(Index),
    WidgetUncapturesKeyboard(Index),
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct MouseDrag {
    pub button: MouseButton,
    pub start: Point,
    pub end: Point,
    pub modifier: ModifierKey,
    pub in_progress: bool,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct MouseClick {
    pub button: MouseButton,
    pub location: Point,
    pub modifier: ModifierKey,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct Scroll {
    pub x: f64,
    pub y: f64,
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

impl RelativePosition for ConrodEvent {
    fn relative_to(&self, xy: Point) -> Self {
        use self::ConrodEvent::{MouseClick, MouseDrag, Raw};
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

impl ConrodEvent {
    pub fn is_mouse_event(&self) -> bool {
        match *self {
            ConrodEvent::Raw(Input::Press(Button::Mouse(_))) => true,
            ConrodEvent::Raw(Input::Release(Button::Mouse(_))) => true,
            ConrodEvent::Raw(Input::Move(Motion::MouseCursor(_, _))) => true,
            ConrodEvent::Raw(Input::Move(Motion::MouseRelative(_, _))) => true,
            ConrodEvent::Raw(Input::Move(Motion::MouseScroll(_, _))) => true,
            ConrodEvent::MouseClick(_) => true,
            ConrodEvent::MouseDrag(_) => true,
            ConrodEvent::Scroll(_) => true,
            _ => false
        }
    }

    pub fn is_keyboard_event(&self) -> bool {
        match *self {
            ConrodEvent::Raw(Input::Press(Button::Keyboard(_))) => true,
            ConrodEvent::Raw(Input::Release(Button::Keyboard(_))) => true,
            ConrodEvent::Raw(Input::Text(_)) => true,
            ConrodEvent::Scroll(_) => true,
            _ => false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use input::{Input, MouseButton, Motion, Button, JoystickAxisArgs};
    use input::keyboard::{self, Key, ModifierKey, NO_MODIFIER};
    use position::Point;

    // We'll see if this approach causes problems later on down the road...
    #[test]
    fn scroll_event_shoulbe_be_both_a_mouse_and_keyboard_event() {
        let scroll_event = ConrodEvent::Scroll(Scroll{
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
            ConrodEvent::Raw(Input::Press(Button::Keyboard(Key::L))),
            ConrodEvent::Raw(Input::Release(Button::Keyboard(Key::L))),
            ConrodEvent::Raw(Input::Text("wha?".to_string())),
        ];
        for event in keyboard_events {
            assert!(event.is_keyboard_event(), format!("{:?} should be a keyboard event", event));
        }

        let non_keyboard_events = vec![
            ConrodEvent::Raw(Input::Press(Button::Mouse(MouseButton::Left))),
            ConrodEvent::Raw(Input::Release(Button::Mouse(MouseButton::Left))),
            ConrodEvent::MouseClick(MouseClick{
                button: MouseButton::Left,
                location: [0.0, 0.0],
                modifier: NO_MODIFIER
            }),
            ConrodEvent::MouseDrag(MouseDrag{
                button: MouseButton::Left,
                start: [0.0, 0.0],
                end: [0.0, 0.0],
                modifier: NO_MODIFIER,
                in_progress: true,
            }),
            ConrodEvent::Raw(Input::Move(Motion::MouseCursor(2.0, 3.0))),
            ConrodEvent::Raw(Input::Move(Motion::MouseRelative(2.0, 3.0))),
            ConrodEvent::Raw(Input::Move(Motion::MouseScroll(3.5, 6.5))),
        ];

        for event in non_keyboard_events {
            assert!(!event.is_keyboard_event(), format!("{:?} should not be a keyboard event", event));
        }
    }

    #[test]
    fn is_mouse_event_should_be_true_for_all_mouse_events() {
        let mouse_events = vec![
            ConrodEvent::Raw(Input::Press(Button::Mouse(MouseButton::Left))),
            ConrodEvent::Raw(Input::Release(Button::Mouse(MouseButton::Left))),
            ConrodEvent::MouseClick(MouseClick{
                button: MouseButton::Left,
                location: [0.0, 0.0],
                modifier: NO_MODIFIER
            }),
            ConrodEvent::MouseDrag(MouseDrag{
                button: MouseButton::Left,
                start: [0.0, 0.0],
                end: [0.0, 0.0],
                modifier: NO_MODIFIER,
                in_progress: true,
            }),
            ConrodEvent::Raw(Input::Move(Motion::MouseCursor(2.0, 3.0))),
            ConrodEvent::Raw(Input::Move(Motion::MouseRelative(2.0, 3.0))),
            ConrodEvent::Raw(Input::Move(Motion::MouseScroll(3.5, 6.5))),
        ];
        for event in mouse_events {
            assert!(event.is_mouse_event(), format!("{:?}.is_mouse_event() == false", event));
        }

        let non_mouse_events = vec![
            ConrodEvent::Raw(Input::Press(Button::Keyboard(Key::G))),
            ConrodEvent::Raw(Input::Release(Button::Keyboard(Key::G))),
            ConrodEvent::Raw(Input::Move(Motion::JoystickAxis(JoystickAxisArgs{
                id: 0,
                axis: 0,
                position: 0f64
            }))),
            ConrodEvent::Raw(Input::Text("rust is brown".to_string())),
            ConrodEvent::Raw(Input::Resize(0, 0)),
            ConrodEvent::Raw(Input::Focus(true)),
            ConrodEvent::Raw(Input::Cursor(true)),
        ];
        for event in non_mouse_events {
            assert!(!event.is_mouse_event(), format!("{:?}.is_mouse_event() == true", event));
        }
    }

    #[test]
    fn mouse_click_should_be_made_relative() {
        let original = ConrodEvent::MouseClick(MouseClick{
            button: MouseButton::Middle,
            location: [30.0, -80.0],
            modifier: keyboard::SHIFT
        });
        let relative = original.relative_to([10.0, 20.0]);

        if let ConrodEvent::MouseClick(click) = relative {
            assert_eq!([20.0, -100.0], click.location);
            assert_eq!(MouseButton::Middle, click.button);
            assert_eq!(keyboard::SHIFT, click.modifier);
        } else {
            panic!("expected a mouse click");
        }
    }

    #[test]
    fn mouse_drage_should_be_made_relative() {
        let original = ConrodEvent::MouseDrag(MouseDrag{
            start: [20.0, 5.0],
            end: [50.0, 1.0],
            button: MouseButton::Left,
            modifier: keyboard::CTRL,
            in_progress: false
        });

        let relative = original.relative_to([-5.0, 5.0]);
        if let ConrodEvent::MouseDrag(drag) = relative {
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
        let original = ConrodEvent::Raw(Input::Move(Motion::MouseCursor(-44.0, 55.0)));
        let relative = original.relative_to([4.0, 5.0]);
        if let ConrodEvent::Raw(Input::Move(Motion::MouseCursor(x, y))) = relative {
            assert_eq!(-48.0, x);
            assert_eq!(50.0, y);
        } else {
            panic!("expected a mouse move event");
        }
    }

    #[test]
    fn mouse_relative_motion_should_be_made_relative() {
        let original = ConrodEvent::Raw(Input::Move(Motion::MouseRelative(-2.0, -4.0)));
        let relative = original.relative_to([3.0, 3.0]);
        if let ConrodEvent::Raw(Input::Move(Motion::MouseRelative(x, y))) = relative {
            assert_eq!(-5.0, x);
            assert_eq!(-7.0, y);
        } else {
            panic!("expected a mouse relative motion event");
        }
    }
}
