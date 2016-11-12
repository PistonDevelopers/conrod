//! Functionality for simplifying the work involved when using conrod along-side piston.

extern crate shader_version;

pub mod draw;
pub mod event;
pub mod window;
pub mod gfx;

pub use self::window::{Window, EventWindow};
pub use self::shader_version::OpenGL;
pub use piston_input::UpdateEvent;

/// This module allows use of the default piston game event loop from `pistoncore_event_loop` with `Window`
pub mod core_events {
    extern crate event_loop;

    pub use self::event_loop::{WindowEvents, EventLoop};
    use super::window::{Window, EventWindow};
    use piston_input::Event;

    impl EventWindow<WindowEvents> for Window {
        fn next(&mut self, events: &mut WindowEvents) -> Option<Event> {
            if let Some(e) = events.next(&mut self.window) {
                self.event(&e);
                Some(e)
            } else { None }
        }
    }
}