#![deny(missing_copy_implementations)]

#[macro_use] extern crate bitflags;
extern crate clock_ticks;
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
pub use ui::Ui;
pub use widget::Widget;

#[macro_use]
pub mod macros;

pub mod background;
pub mod callback;
pub mod color;
pub mod dimensions;
pub mod draw;
pub mod frame;
pub mod label;
pub mod mouse;
pub mod point;
pub mod position;
pub mod rectangle;
pub mod shape;
mod theme;
pub mod ui;
pub mod utils;
pub mod widget;
