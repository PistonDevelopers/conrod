//! Handles all of the global input events and state.
//! The core of this module is the `Global` struct. It is responsible for aggregating
//! and interpreting raw input events into high-level semantic events.

use event;
use input;
use std;

/// Global input event handler that also implements `input::Provider`. The `Ui` passes all events
/// to it's `Global` instance, which aggregates and interprets the events to provide so-called
/// 'high-level' events to widgets. This input gets reset after every update by the `Ui`.
#[derive(Debug)]
pub struct Global {
    /// The `input::State` as it was at the end of the last update cycle.
    pub start: input::State,
    /// The most recent `input::State`, with updates from handling all the events
    /// this update cycle
    pub current: input::State,
    /// The events that have occurred between two consecutive updates.
    events: Vec<event::Event>,
    /// Tracks the last click that occurred and the time at which it occurred in order to create
    /// double-click events.
    pub last_click: Option<(instant::Instant, event::Click)>,
}

/// Iterator over all global `event::Event`s that have occurred since the last time
/// `Ui::set_widgets` was called.
#[derive(Clone)]
pub struct Events<'a> {
    iter: std::slice::Iter<'a, event::Event>,
}

/// An iterator yielding all `event::Ui`s that have occurred since the last time `Ui::set_widgets`
/// was called.
#[derive(Clone)]
pub struct UiEvents<'a> {
    events: Events<'a>,
}

impl Global {

    /// Returns a fresh new `Global`
    pub fn new() -> Global {
        Global{
            events: Vec::new(),
            start: input::State::new(),
            current: input::State::new(),
            last_click: None,
        }
    }

    /// Returns an iterator yielding all events that have occurred since the last time
    /// `Ui::set_widgets` was called.
    pub fn events(&self) -> Events {
        Events { iter: self.events.iter() }
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

impl<'a> Events<'a> {
    /// Converts the `Events` into a `UiEvents`, yielding only the `event::Ui`s that have occurred
    /// since the last time `Ui::set_widgets` was called.
    pub fn ui(self) -> UiEvents<'a> {
        UiEvents { events: self }
    }
}

impl<'a> Iterator for Events<'a> {
    type Item = &'a event::Event;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a> Iterator for UiEvents<'a> {
    type Item = &'a event::Ui;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(event) = self.events.next() {
            if let event::Event::Ui(ref ui_event) = *event {
                return Some(ui_event);
            }
        }
        None
    }
}
