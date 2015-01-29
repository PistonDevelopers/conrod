
#![feature(slicing_syntax)]
#![deny(missing_copy_implementations)]
#![allow(unstable)]

#[macro_use] extern crate bitflags;
extern crate input;
extern crate event;
extern crate freetype;
extern crate graphics;
extern crate opengl_graphics;
extern crate "rustc-serialize" as rustc_serialize;
extern crate clock_ticks;
extern crate vecmath;

pub use background::Background;
pub use button::Button;
pub use drop_down_list::DropDownList;
pub use envelope_editor::EnvelopeEditor;
pub use envelope_editor::EnvelopePoint;
pub use label::Label;
pub use number_dialer::NumberDialer;
pub use slider::Slider;
pub use text_box::TextBox;
pub use toggle::Toggle;
pub use widget_matrix::WidgetMatrix;
pub use xy_pad::XYPad;

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
pub use ui_context::UiContext;
pub use widget::Widget;

#[macro_use]
pub mod macros;

pub mod background;
pub mod button;
pub mod callback;
pub mod color;
pub mod dimensions;
pub mod draw;
pub mod drop_down_list;
pub mod envelope_editor;
pub mod frame;
pub mod label;
pub mod mouse;
pub mod number_dialer;
pub mod point;
pub mod position;
pub mod rectangle;
pub mod shape;
pub mod slider;
pub mod text_box;
pub mod theme;
pub mod toggle;
pub mod ui_context;
pub mod utils;
pub mod widget;
pub mod widget_matrix;
pub mod xy_pad;
