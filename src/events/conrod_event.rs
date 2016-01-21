use input::{Input, MouseButton, Motion};
use input::keyboard::ModifierKey;
use position::Point;
use vecmath::vec2_sub;

#[derive(Clone, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum ConrodEvent {
    Raw(Input),
    MouseClick(MouseClick),
    MouseDrag(MouseDrag),
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

#[cfg(test)]
mod test {
    use super::*;
    use input::{Input, MouseButton, Motion};
    use input::keyboard::{self, ModifierKey};
    use position::Point;

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
