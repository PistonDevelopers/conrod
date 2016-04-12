//! Handles all of the global input events and state.
//! The core of this module is the `Global` struct. It is responsible for aggregating
//! and interpreting raw input events into high-level semantic events.

use event;
use input;

/// Global input event handler that also implements `input::Provider`. The `Ui` passes all events
/// to it's `Global` instance, which aggregates and interprets the events to provide so-called
/// 'high-level' events to widgets. This input gets reset after every update by the `Ui`.
pub struct Global {
    /// The `input::State` as it was at the end of the last update cycle.
    pub start: input::State,
    /// The most recent `input::State`, with updates from handling all the events
    /// this update cycle
    pub current: input::State,
    /// The events that have occurred between two consecutive updates.
    events: Vec<event::Event>,
}

/// Iterator over global `event::Event`s.
///
/// Unlike the `input::widget::Events`, this will never filter out any events, and all coordinates
/// will be relative to the middle of the window.
pub type Events<'a> = ::std::slice::Iter<'a, event::Event>;

impl Global {

    /// Returns a fresh new `Global`
    pub fn new() -> Global {
        Global{
            events: Vec::new(),
            start: input::State::new(),
            current: input::State::new(),
        }
    }

    /// Returns an iterator yielding all events that have occurred since the last time
    /// `Ui::set_widgets` was called.
    pub fn events(&self) -> Events {
        self.events.iter()
    }

    /// Add the new event to the stack.
    pub fn push_event(&mut self, event: event::Event) {
        self.events.push(event);
    }

    /// Called at the end of every update cycle in order to prepare the `Global` to
    /// handle events for the next one.
    pub fn clear_events_and_update_start_state(&mut self) {
        self.events.clear();
        self.start = self.current.clone();
    }

}
