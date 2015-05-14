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


pub use canvas::split::Split;
pub use canvas::floating::Floating;

pub use widget::button::Button;
pub use widget::drop_down_list::DropDownList;
pub use widget::envelope_editor::EnvelopeEditor;
pub use widget::envelope_editor::EnvelopePoint;
pub use widget::label::Label;
pub use widget::matrix::Matrix as WidgetMatrix;
pub use widget::number_dialer::NumberDialer;
pub use widget::slider::Slider;
pub use widget::text_box::TextBox;
pub use widget::toggle::Toggle;
pub use widget::xy_pad::XYPad;

pub use background::Background;
pub use canvas::{Canvas, CanvasId};
pub use elmesque::color;
pub use elmesque::color::{Color, Colorable};
pub use frame::{Framing, Frameable};
pub use graphics::character::CharacterCache;
pub use label::{FontSize, Labelable};
pub use mouse::Mouse;
pub use mouse::ButtonState as MouseButtonState;
pub use position::{align_left_of, align_right_of, align_bottom_of, align_top_of};
pub use position::{middle_of, top_left_of, top_right_of, bottom_left_of, bottom_right_of,
                   mid_top_of, mid_bottom_of, mid_left_of, mid_right_of};
pub use position::{Corner, Depth, Direction, Dimensions, HorizontalAlign, Place, Point, Position,
                   Positionable, Sizeable, VerticalAlign};
pub use theme::{Align, Theme};
pub use ui::{GlyphCache, Ui, UiId, UserInput};
pub use widget::{Widget, WidgetId};


mod background;
mod canvas;
mod frame;
mod label;
mod mouse;
mod position;
mod theme;
mod ui;
pub mod utils;
mod widget;

