//! 
//! # Conrod
//!
//! An easy-to-use, immediate-mode, 2D GUI library featuring a range of useful widgets.
//!

#![deny(missing_copy_implementations)]
#![warn(missing_docs)]

#[macro_use] extern crate bitflags;
extern crate clock_ticks;
extern crate elmesque;
extern crate graphics;
#[macro_use] extern crate piston;
extern crate rand;
extern crate rustc_serialize;
extern crate vecmath;
extern crate num;

pub use widget::button::Button;
pub use widget::drop_down_list::DropDownList;
pub use widget::envelope_editor::EnvelopeEditor;
pub use widget::envelope_editor::EnvelopePoint;
pub use widget::label::Label;
pub use widget::number_dialer::NumberDialer;
pub use widget::slider::Slider;
pub use widget::text_box::TextBox;
pub use widget::toggle::Toggle;
pub use widget::matrix::Matrix as WidgetMatrix;
pub use widget::xy_pad::XYPad;

pub use background::Background;
pub use callback::Callable;
pub use color::{Color, Colorable};
pub use dimensions::Dimensions;
pub use draw::Drawable;
pub use frame::{Framing, Frameable};
pub use label::Labelable;
pub use point::Point;
pub use position::Positionable;
pub use shape::Shapeable;
pub use theme::Theme;
pub use ui::{Ui, UiId};

#[macro_use]
mod macros;

mod background;
mod callback;
pub mod color;
mod dimensions;
mod draw;
mod frame;
mod label;
mod mouse;
mod point;
mod position;
mod rectangle;
mod shape;
mod theme;
mod ui;
mod utils;
mod widget;
