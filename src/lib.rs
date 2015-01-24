
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
extern crate quack;

pub use background::BackgroundBuilder as Background;
pub use button::ButtonBuilder as Button;
pub use drop_down_list::DropDownListBuilder as DropDownList;
pub use envelope_editor::EnvelopeEditorBuilder as EnvelopeEditor;
pub use envelope_editor::EnvelopePoint;
pub use label::LabelBuilder as Label;
pub use number_dialer::NumberDialerBuilder as NumberDialer;
pub use slider::SliderBuilder as Slider;
pub use text_box::TextBoxBuilder as TextBox;
pub use toggle::ToggleBuilder as Toggle;
pub use widget_matrix::WidgetMatrixBuilder as WidgetMatrix;
pub use xy_pad::XYPadBuilder as XYPad;

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

pub mod internal;
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

#[derive(Copy)]
pub struct FontSize(pub internal::FontSize);

/// Point property.
#[derive(Copy)]
pub struct Position(pub internal::Point);
