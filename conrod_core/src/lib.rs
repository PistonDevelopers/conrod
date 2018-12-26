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

pub use crate::{
    color::{Color, Colorable},
    conrod_derive::*,
    border::{Bordering, Borderable},
    label::{FontSize, Labelable},
    position::{Dimensions, Point, Position, Positionable, Range, Rect, Scalar, Sizeable},
    theme::Theme,
    ui::{Ui, UiCell, UiBuilder},
    widget::{scroll, Widget}
};

pub mod color;
pub mod event;
pub mod graph;
pub mod guide;
pub mod image;
pub mod input;
pub mod position;
pub mod render;
pub mod text;
pub mod theme;
pub mod utils;
pub mod widget;
pub mod cursor;
pub mod border;
pub mod label;
pub mod ui;

#[cfg(test)] mod tests;
