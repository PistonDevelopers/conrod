//! Functionality for simplifying the work involved when using conrod along-side piston.

pub mod draw;
pub mod event;
pub mod window;
pub mod gfx;
pub use self::window::Window;
pub use piston_input::UpdateEvent;

/// This module allows use of the default piston game event loop from `pistoncore_event_loop` with `Window`
pub mod core_event_loop {
    extern crate event_loop;

    pub use self::event_loop::{WindowEvents, EventLoop};
    use super::window::{Window, EventWindow};
    use piston_input::Event;

    impl EventWindow for WindowEvents {
        fn next(&mut self, window: &mut Window) -> Option<Event> {
            if let Some(e) = self.next(window) {
                window.event(&e);
                Some(e)
            } else { None }
        }
    }
}