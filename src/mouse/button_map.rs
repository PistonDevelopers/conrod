use super::MouseButton;
use super::ButtonState;
use std::slice::{IterMut, Iter};
use position::Point;

/// The max total number of buttons on a mouse.
pub const NUM_MOUSE_BUTTONS: usize = 9;

/// A map of `conrod::MouseButton` to `conrod::MouseButtonState`.
/// Used by the `conrod::Mouse` to hold the state of all mouse buttons.
#[derive(Copy, Clone, Debug)]
pub struct ButtonMap {
    button_states: [ButtonState; NUM_MOUSE_BUTTONS]
}

impl ButtonMap {
    /// Returns a new button map with all states set to defaults.
    pub fn new() -> ButtonMap {
        ButtonMap{
            button_states: [ButtonState::new(); NUM_MOUSE_BUTTONS]
        }
    }

    /// Returns an immutable reference to the button State
    /// for the given button.
    ///
    /// # Example
    /// ```
    /// use conrod::{MouseButton, MouseButtonMap};
    ///
    /// let map = MouseButtonMap::new();
    /// let left_button_state = map.get(MouseButton::Left);
    /// assert!(left_button_state.is_up());
    /// ```
    pub fn get(&self, button: MouseButton) -> &ButtonState {
        &self.button_states[Self::button_idx(button)]
    }

    /// Returns a mutable reference to the button state for the given button.
    ///
    /// # Example
    /// ```
    /// use conrod::{MouseButton, MouseButtonMap};
    ///
    /// let mut map = MouseButtonMap::new();
    /// {
    ///     let right_button_state = map.get_mut(MouseButton::Right);
    ///     right_button_state.was_just_pressed = true;
    /// }
    /// assert!(map.get(MouseButton::Right).was_just_pressed);
    /// ```
    pub fn get_mut(&mut self, button: MouseButton) -> &mut ButtonState {
        &mut self.button_states[Self::button_idx(button)]
    }

    /// Simple way to update the ButtonState for a given MouseButton using a closure.
    ///
    /// # Example
    /// ```
    /// use conrod::{MouseButton, MouseButtonState, MouseButtonMap};
    ///
    /// let mut map = MouseButtonMap::new();
    /// map.update(MouseButton::Right, |state| state.was_just_released = true);
    /// assert!(map.get(MouseButton::Right).was_just_released);
    /// ```
    pub fn update<F: FnOnce(&mut ButtonState)>(&mut self, button: MouseButton, update_fn: F) {
        let state = self.get_mut(button);
        update_fn(state);
    }

    /// Returns a `Vec` containing all the button states in the map, combined
    /// with their respective buttons.
    ///
    /// # Example
    /// ```
    /// use conrod::{MouseButton, NUM_MOUSE_BUTTONS, MouseButtonState, MouseButtonMap};
    ///
    /// let mut map = MouseButtonMap::new();
    /// let button_states: Vec<(MouseButton, &MouseButtonState)> = map.all_buttons();
    /// assert_eq!(NUM_MOUSE_BUTTONS, button_states.len());
    /// ```
    pub fn all_buttons(&self) -> Vec<(MouseButton, &ButtonState)> {
        self.button_states.into_iter().enumerate()
            .map(|idx_and_state| {
                (MouseButton::from(idx_and_state.0 as u32), idx_and_state.1)
            }).collect::<Vec<(MouseButton, &ButtonState)>>()
    }

    /// returns an iterator over mutable references to all the button states
    pub fn iter_mut(&mut self) -> IterMut<ButtonState> {
        self.button_states.iter_mut()
    }

    /// returns an iterator over immutable references to all the button states
    pub fn iter(&self) -> Iter<ButtonState> {
        self.button_states.iter()
    }

    /// Returns a new `ButtonMap` with all points relative to the
    /// given `Point`.
    pub fn relative_to(&self, xy: Point) -> ButtonMap {
        let mut new_map = self.clone();
        for mut state in new_map.button_states.iter_mut() {
            state.event = state.event.map(|evt| evt.relative_to(xy));
        }
        new_map
    }

    fn button_idx(button: MouseButton) -> usize {
        u32::from(button) as usize
    }
}

#[test]
fn relative_to_changes_all_mouse_events_to_relative_positions() {
    use input::MouseButton::*;
    use super::simple_events::*;
    use super::simple_events::SimpleMouseEvent::{Click, Drag};
    use time::SteadyTime;

    let mut map_a = ButtonMap::new();
    map_a.update(Left, |state| state.event = Some(Click(MouseClick{
        mouse_button: Left,
        position: [10.0, 50.5]
    })));
    map_a.update(Button6, |state| state.event = Some(Drag(MouseDragEvent{
        mouse_button: Button6,
        start: MouseButtonDown{
            time: SteadyTime::now(),
            position: [20.2, 30.0]
        },
        current: MouseButtonDown{
            time: SteadyTime::now(),
            position: [40.4, 60.0]
        },
        button_released: true
    })));

    let map_b = map_a.relative_to([10.0, 10.0]);

    let left_state = map_b.get(Left);
    if let Some(Click(click)) = left_state.event {
        assert_float_eq(0.0, click.position[0]);
        assert_float_eq(40.5, click.position[1]);
    } else {
        panic!("Expected click event, got: {:?}", left_state.event);
    }

    let state_6 = map_b.get(Button6);
    if let Some(Drag(drag)) = state_6.event {
        assert_float_eq(10.2, drag.start.position[0]);
        assert_float_eq(20.0, drag.start.position[1]);
        assert_float_eq(30.4, drag.current.position[0]);
        assert_float_eq(50.0, drag.current.position[1]);
    } else {
        panic!("Expected a drag event, but got: {:?}", state_6.event);
    }
}
