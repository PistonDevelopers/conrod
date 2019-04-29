//! Everything related to storing the state of user input.
//!
//! This includes the state of any buttons on either the keyboard or the mouse, as well as the
//! position of the mouse.
//!
//! It also includes which widgets, if any, are capturing the keyboard and mouse.
//!
//! This module exists mostly to support the `input::Provider` trait.

use position::Point;
use self::mouse::Mouse;
use fnv;
use super::keyboard::ModifierKey;
use utils;
use widget;


/// Holds the current state of user input.
///
/// This includes the state of all buttons on the keyboard and mouse, as well as the position of
/// the mouse.
///
/// It also includes which widgets, if any, are capturing keyboard and mouse input.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// Mouse position and button state.
    pub mouse: Mouse,
    /// All in-progress touch interactions.
    pub touch: fnv::FnvHashMap<super::touch::Id, self::touch::Touch>,
    /// Which widget, if any, is currently capturing the keyboard
    pub widget_capturing_keyboard: Option<widget::Id>,
    /// Which widget, if any, is currently capturing the mouse
    pub widget_capturing_mouse: Option<widget::Id>,
    /// The widget that is currently under the mouse cursor.
    ///
    /// If the mouse is currently over multiple widgets, this index will represent the top-most,
    /// non-graphic-child widget.
    pub widget_under_mouse: Option<widget::Id>,
    /// Which modifier keys are being held down.
    pub modifiers: ModifierKey,
}

impl State {

    /// Returns a fresh new input state
    pub fn new() -> State {
        State{
            touch: fnv::FnvHashMap::default(),
            mouse: Mouse::new(),
            widget_capturing_keyboard: None,
            widget_capturing_mouse: None,
            widget_under_mouse: None,
            modifiers: ModifierKey::NO_MODIFIER,
        }
    }

    /// Returns a copy of the input::State relative to the given `position::Point`
    pub fn relative_to(mut self, xy: Point) -> State {
        self.mouse.xy = utils::vec2_sub(self.mouse.xy, xy);
        self.mouse.buttons = self.mouse.buttons.relative_to(xy);
        self
    }

}

/// Touch specific state.
pub mod touch {
    use position::Point;
    use widget;

    /// State stored about the start of a `Touch` interaction.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Start {
        /// The time at which the `Touch` began.
        pub time: instant::Instant,
        /// The position at which the touch began.
        pub xy: Point,
        /// The widget under the beginning of the touch if there was one.
        ///
        /// This widget captures the `Touch` input source for its duration.
        pub widget: Option<widget::Id>,
    }

    /// All state stored for a `Touch` interaction in progress.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Touch {
        /// The `Start` of the touch interaction.
        pub start: Start,
        /// The last recorded position of the finger on the window.
        pub xy: Point,
        /// The widget currently being touched.
        pub widget: Option<widget::Id>,
    }

}

/// Mouse specific state.
pub mod mouse {
    use position::Point;
    use std;
    use widget;

    #[doc(inline)]
    pub use input::MouseButton as Button;

    /// The max total number of buttons on a mouse.
    pub const NUM_BUTTONS: usize = 9;

    /// The state of the `Mouse`, including it's position and button states.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Mouse {
        /// A map that stores the up/down state of each button.
        ///
        /// If the button is down, then it stores the position of the mouse when the button was first
        /// pressed.
        pub buttons: ButtonMap,
        /// The current position of the mouse.
        pub xy: Point,
    }

    /// Whether the button is up or down.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum ButtonPosition {
        /// The button is up (i.e. pressed).
        Up,
        /// The button is down and was originally pressed down at the given `Point` over the widget
        /// at the given widget::Id.
        Down(Point, Option<widget::Id>),
    }

    /// Stores the state of all mouse buttons.
    ///
    /// If the mouse button is down, it stores the position of the mouse when the button was pressed
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct ButtonMap {
        buttons: [ButtonPosition; NUM_BUTTONS]
    }

    /// An iterator yielding all pressed buttons.
    #[derive(Clone)]
    pub struct PressedButtons<'a> {
        buttons: ::std::iter::Enumerate<::std::slice::Iter<'a, ButtonPosition>>,
    }

    impl Mouse {
        /// Construct a new default `Mouse`.
        pub fn new() -> Self {
            Mouse {
                buttons: ButtonMap::new(),
                xy: [0.0, 0.0],
            }
        }
    }

    impl ButtonPosition {

        /// If the mouse button is down, return a new one with position relative to the given `xy`.
        pub fn relative_to(self, xy: Point) -> Self {
            match self {
                ButtonPosition::Down(pos, widget) =>
                    ButtonPosition::Down([pos[0] - xy[0], pos[1] - xy[1]], widget),
                button_pos => button_pos,
            }
        }

        /// Is the `ButtonPosition` down.
        pub fn is_down(&self) -> bool {
            match *self {
                ButtonPosition::Down(_, _) => true,
                _ => false,
            }
        }

        /// Is the `ButtonPosition` up.
        pub fn is_up(&self) -> bool {
            match *self {
                ButtonPosition::Up => true,
                _ => false,
            }
        }

        /// Returns the position at which the button was pressed along with the widget that was
        /// under the mouse at the time of pressing if the position is `Down`.
        pub fn if_down(&self) -> Option<(Point, Option<widget::Id>)> {
            match *self {
                ButtonPosition::Down(xy, widget) => Some((xy, widget)),
                _ => None,
            }
        }

        /// Returns the position at which the button was pressed if it's down.
        pub fn xy_if_down(&self) -> Option<Point> {
            match *self {
                ButtonPosition::Down(xy, _) => Some(xy),
                _ => None,
            }
        }

    }

    impl ButtonMap {

        /// Returns a new button map with all states set to `None`
        pub fn new() -> Self {
            ButtonMap{
                buttons: [ButtonPosition::Up; NUM_BUTTONS]
            }
        }

        /// Returns a copy of the ButtonMap relative to the given `Point`
        pub fn relative_to(self, xy: Point) -> Self {
            self.buttons.iter().enumerate().fold(ButtonMap::new(), |mut map, (idx, button_pos)| {
                map.buttons[idx] = button_pos.relative_to(xy);
                map
            })
        }

        /// The state of the left mouse button.
        pub fn left(&self) -> &ButtonPosition {
            &self[Button::Left]
        }

        /// The state of the middle mouse button.
        pub fn middle(&self) -> &ButtonPosition {
            &self[Button::Middle]
        }

        /// The state of the right mouse button.
        pub fn right(&self) -> &ButtonPosition {
            &self[Button::Right]
        }

        /// Sets the `Button` in the `Down` position.
        pub fn press(&mut self, button: Button, xy: Point, widget: Option<widget::Id>) {
            self.buttons[button_to_idx(button)] = ButtonPosition::Down(xy, widget);
        }

        /// Set's the `Button` in the `Up` position.
        pub fn release(&mut self, button: Button) {
            self.buttons[button_to_idx(button)] = ButtonPosition::Up;
        }

        /// An iterator yielding all pressed mouse buttons along with the location at which they
        /// were originally pressed.
        pub fn pressed(&self) -> PressedButtons {
            PressedButtons { buttons: self.buttons.iter().enumerate() }
        }

    }

    /// Converts a `Button` to its respective index within the `ButtonMap`.
    fn button_to_idx(button: Button) -> usize {
        let idx: u32 = button.into();
        idx as usize
    }

    /// Converts a `ButtonMap` index to its respective `Button`.
    fn idx_to_button(idx: usize) -> Button {
        (idx as u32).into()
    }

    impl std::ops::Index<Button> for ButtonMap {
        type Output = ButtonPosition;
        fn index(&self, button: Button) -> &Self::Output {
            &self.buttons[button_to_idx(button)]
        }
    }

    impl<'a> Iterator for PressedButtons<'a> {
        type Item = (Button, Point, Option<widget::Id>);
        fn next(&mut self) -> Option<Self::Item> {
            while let Some((idx, button_pos)) = self.buttons.next() {
                if let ButtonPosition::Down(xy, widget) = *button_pos {
                    return Some((idx_to_button(idx), xy, widget));
                }
            }
            None
        }
    }

}



#[test]
fn pressed_next_returns_none_if_no_buttons_are_pressed() {
    let map = mouse::ButtonMap::new();
    let pressed = map.pressed().next();
    assert!(pressed.is_none());
}

#[test]
fn pressed_next_should_return_first_pressed_button() {
    let mut map = mouse::ButtonMap::new();

    map.press(mouse::Button::Right, [3.0, 3.0], None);
    map.press(mouse::Button::X1, [5.4, 4.5], None);

    let pressed = map.pressed().next();
    assert_eq!(Some((mouse::Button::Right, [3.0, 3.0], None)), pressed);
}

#[test]
fn button_down_should_store_the_point() {
    let mut map = mouse::ButtonMap::new();
    let xy = [2.0, 5.0];
    map.press(mouse::Button::Left, xy, None);

    assert_eq!(mouse::ButtonPosition::Down(xy, None), map[mouse::Button::Left]);
}

#[test]
fn input_state_should_be_made_relative_to_a_given_point() {
    let mut state = State::new();
    state.mouse.xy = [50.0, -10.0];
    state.mouse.buttons.press(mouse::Button::Middle, [-20.0, -10.0], None);

    let relative_state = state.relative_to([20.0, 20.0]);
    assert_eq!([30.0, -30.0], relative_state.mouse.xy);
    assert_eq!(Some([-40.0, -30.0]), relative_state.mouse.buttons[mouse::Button::Middle].xy_if_down());
}
