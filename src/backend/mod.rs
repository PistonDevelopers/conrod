//! Backend-specific implementations.
//!
//! Each module is prefixed by either `event` or `draw`.
//!
//! Those prefixed with `event` contain functions for converting events polled from their window
//! backend to conrod's `event::Raw` type.
//!
//! Those prefixed with `draw` contain functions for rendering conrod's `Primitives` iterator to
//! to some graphics backend.

pub mod draw_piston;
pub mod event_piston;
