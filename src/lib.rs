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
extern crate num;
#[macro_use] extern crate piston;
extern crate rand;
extern crate rustc_serialize;
extern crate vecmath;


pub use widget::button::Button;
pub use widget::custom::Custom as CustomWidget;
pub use widget::custom::State as CustomWidgetState;
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
pub use elmesque::color;
pub use elmesque::color::{Color, Colorable};
pub use frame::{Framing, Frameable};
pub use label::Labelable;
pub use mouse::Mouse;
pub use mouse::ButtonState as MouseButtonState;
pub use position::{align_left_of, align_right_of, align_bottom_of, align_top_of};
pub use position::{Corner, Depth, Direction, Dimensions, HorizontalAlign, Point, Position,
                   Positionable, Sizeable, VerticalAlign};
pub use theme::Theme;
pub use ui::{Ui, UiId};

#[macro_use]
mod macros;

mod background;
mod frame;
mod label;
pub mod mouse;
mod position;
mod theme;
mod ui;
mod utils;
mod widget;

