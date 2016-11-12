//! Functionality for simplifying the work involved when using conrod along-side piston.

extern crate shader_version;

pub mod draw;
pub mod event;
pub mod window;
pub mod gfx;

pub use self::window::{Window, EventWindow};
pub use self::shader_version::OpenGL;
