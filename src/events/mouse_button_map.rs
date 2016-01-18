use input::MouseButton;
use std::slice::{IterMut, Iter};
use position::Point;

/// The max total number of buttons on a mouse.
pub const NUM_MOUSE_BUTTONS: usize = 9;

pub type ButtonDownPosition = Option<Point>;

#[derive(Copy, Clone, Debug)]
pub struct ButtonMap {
    button_states: [ButtonDownPosition; NUM_MOUSE_BUTTONS]
}

impl ButtonMap {
    /// Returns a new button map with all states set to defaults.
    pub fn new() -> ButtonMap {
        ButtonMap{
            button_states: [None; NUM_MOUSE_BUTTONS]
        }
    }

    pub fn set(&mut self, button: MouseButton, point: ButtonDownPosition) {
        let idx = ButtonMap::button_idx(button);
        self.button_states[idx] = point;
    }

    pub fn get(&self, button: MouseButton) -> ButtonDownPosition {
        self.button_states[ButtonMap::button_idx(button)]
    }

    pub fn take(&mut self, button: MouseButton) -> ButtonDownPosition {
        self.button_states[ButtonMap::button_idx(button)].take()
    }

    pub fn pressed_button(&self) -> Option<(MouseButton, Point)> {
        self.button_states.iter().enumerate().filter(|idx_and_state| idx_and_state.1.is_some())
                .map(|idx_and_state|
                    (ButtonMap::idx_to_button(idx_and_state.0), idx_and_state.1.unwrap()))
                .next()
    }

    fn idx_to_button(idx: usize) -> MouseButton {
        MouseButton::from(idx as u32)
    }
    fn button_idx(button: MouseButton) -> usize {
        u32::from(button) as usize
    }

}

#[test]
fn pressed_button_returns_none_if_no_buttons_are_pressed() {
    let mut map = ButtonMap::new();
    let pressed = map.pressed_button();
    assert!(pressed.is_none());
}

#[test]
fn pressed_button_should_return_first_pressed_button() {
    let mut map = ButtonMap::new();

    map.set(MouseButton::Right, Some([3.0, 3.0]));
    map.set(MouseButton::X1, Some([5.4, 4.5]));

    let pressed = map.pressed_button();
    assert_eq!(Some((MouseButton::Right, [3.0, 3.0])), pressed);
}

#[test]
fn button_down_should_store_the_point() {
    let mut map = ButtonMap::new();
    let point: ButtonDownPosition = Some([2.0, 5.0]);
    map.set(MouseButton::Left, point);

    assert_eq!(point, map.get(MouseButton::Left));
}

#[test]
fn take_resets_and_returns_current_state() {
    let mut map = ButtonMap::new();
    let point: ButtonDownPosition = Some([2.0, 5.0]);
    map.set(MouseButton::Left, point);

    let taken = map.take(MouseButton::Left);
    assert_eq!(point, taken);
    assert!(map.get(MouseButton::Left).is_none());
}
