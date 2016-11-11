//! # Conrod
//!
//! An easy-to-use, immediate-mode, 2D GUI library featuring a range of useful widgets.
//!
//! If you are new to Conrod, we recommend checking out [The Guide](./guide/index.html).

#![deny(missing_copy_implementations)]
#![warn(missing_docs)]

extern crate daggy;
extern crate num;
extern crate input as piston_input;
extern crate rusttype;

#[cfg(feature="glium")] #[macro_use] pub extern crate glium;

pub use color::{Color, Colorable};
pub use border::{Bordering, Borderable};
pub use label::{FontSize, Labelable};
pub use position::{Align, Axis, Corner, Depth, Direction, Dimension, Dimensions, Edge, Margin,
                   Padding, Place, Point, Position, Positionable, Range, Rect, Scalar, Sizeable};
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
mod position;
pub mod render;
pub mod text;
pub mod theme;
mod ui;
pub mod utils;
pub mod widget;

#[cfg(test)] mod tests;
