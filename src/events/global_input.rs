//! Handles all of the global input events and state.
//! The core of this module is the `GlobalInput` struct. It is responsible for aggregating
//! and interpreting raw input events into high-level semantic events.

use events::{InputState, UiEvent, InputProvider};
use input::MouseButton;
use position::Point;
use widget::Index;

/// Global input event handler that also implements `InputProvider`. The `Ui` passes all events
/// to it's `GlobalInput` instance, which aggregates and interprets the events to provide
/// so-called 'high-level' events to widgets. This input gets reset after every update by the `Ui`.
pub struct GlobalInput {
    /// The `InputState` as it was at the end of the last update cycle.
    pub start: InputState,
    /// The most recent `InputState`, with updates from handling all the events
    /// this update cycle
    pub current: InputState,
    /// The events that have occurred between two consecutive updates.
    events: Vec<UiEvent>,
}

/// Iterator over global `UiEvent`s. Unlike the `WidgetInputEventIterator`, this will
/// never filter out any events, and all coordinates will be reative to the (0,0) origin
/// of the window.
pub type GlobalInputEventIterator<'a> = ::std::slice::Iter<'a, UiEvent>;

impl<'a> InputProvider<'a> for GlobalInput {
    type Events = GlobalInputEventIterator<'a>;

    fn events(&'a self) -> Self::Events {
        self.events.iter()
    }

    fn current(&'a self) -> &'a InputState {
        &self.current
    }

    fn mouse_button_down(&self, button: MouseButton) -> Option<Point> {
        // TODO: Review this as it changes behaviour...
        self.current().mouse.buttons[button].xy_if_down()
        //  self.current().mouse.buttons.get(button).map(|_| {
        //      self.mouse_position()
        //  })
    }
}

impl GlobalInput {

    /// Returns a fresh new `GlobalInput`
    pub fn new() -> GlobalInput {
        GlobalInput{
            events: Vec::new(),
            start: InputState::new(),
            current: InputState::new(),
        }
    }

    /// Add the new event to the stack.
    pub fn push_event(&mut self, event: UiEvent) {
        self.events.push(event);
    }

    /// Called at the end of every update cycle in order to prepare the `GlobalInput` to
    /// handle events for the next one.
    pub fn clear_events_and_update_start_state(&mut self) {
        self.events.clear();
        self.start = self.current.clone();
    }

    /// Returns the most up to date position of the mouse
    pub fn mouse_position(&self) -> Point {
        self.current.mouse.xy
    }

    /// Returns the input state as it was after the last update
    pub fn starting_state(&self) -> &InputState {
        &self.start
    }

    /// Returns the most up to date info on which widget is capturing the mouse
    pub fn currently_capturing_mouse(&self) -> Option<Index> {
        self.current.widget_capturing_mouse
    }

    /// Returns the most up to date info on which widget is capturing the keyboard
    pub fn currently_capturing_keyboard(&self) -> Option<Index> {
        self.current.widget_capturing_keyboard
    }

}
