//! # Conrod
//!
//! An easy-to-use, immediate-mode, 2D GUI library featuring a range of useful widgets.
//!
//! If you are new to Conrod, we recommend checking out [The Guide](./guide/index.html).

#![deny(unsafe_code)]
#![deny(missing_copy_implementations)]
#![warn(missing_docs)]

#[macro_use] extern crate conrod_derive;
extern crate daggy;
extern crate fnv;
extern crate num;
extern crate input as piston_input;
extern crate rusttype;

#[cfg(feature="glium")] #[macro_use] pub extern crate glium;
#[cfg(feature="sdl2")] pub extern crate sdl2;
#[cfg(feature="gfx_rs")] #[macro_use] pub extern crate gfx;
#[cfg(feature="gfx_rs")] pub extern crate gfx_core;

pub use color::{Color, Colorable};
pub use border::{Bordering, Borderable};
pub use label::{FontSize, Labelable};
pub use position::{Dimensions, Point, Position, Positionable, Range, Rect, Scalar, Sizeable};
pub use theme::Theme;
pub use ui::{Ui, UiCell, UiBuilder};
pub use widget::{scroll, Widget};

pub mod backend;
mod border;
pub mod color;
pub mod event;
pub mod graph;
pub mod guide;
pub mod image;
pub mod input;
mod label;
pub mod position;
pub mod render;
pub mod text;
pub mod theme;
mod ui;
pub mod utils;
pub mod widget;
pub mod cursor;

#[cfg(test)] mod tests;
