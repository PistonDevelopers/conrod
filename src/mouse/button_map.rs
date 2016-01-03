use super::MouseButton;
use super::ButtonState;
use std::slice::{IterMut, Iter};

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

    pub fn iter(&self) -> Iter<ButtonState> {
        self.button_states.iter()
    }

    fn button_idx(button: MouseButton) -> usize {
        u32::from(button) as usize
    }
}
