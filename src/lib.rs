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
extern crate json_io;
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

pub use widget::button::Style as ButtonStyle;
pub use widget::drop_down_list::Style as DropDownListStyle;
pub use widget::envelope_editor::Style as EnvelopeEditorStyle;
pub use widget::label::Style as LabelStyle;
pub use widget::number_dialer::Style as NumberDialerStyle;
pub use widget::slider::Style as SliderStyle;
pub use widget::text_box::Style as TextBoxStyle;
pub use widget::toggle::Style as ToggleStyle;
pub use widget::xy_pad::Style as XYPadStyle;


pub use background::Background;
pub use canvas::{Canvas, CanvasId};
pub use elmesque::color;
pub use elmesque::color::{Color, Colorable};
pub use frame::{Framing, Frameable};
pub use graphics::character::CharacterCache;
pub use label::{FontSize, Labelable};
pub use mouse::Mouse;
pub use mouse::ButtonState as MouseButtonState;
pub use mouse::Scroll as MouseScroll;
pub use position::{align_left_of, align_right_of, align_bottom_of, align_top_of};
pub use position::{middle_of, top_left_of, top_right_of, bottom_left_of, bottom_right_of,
                   mid_top_of, mid_bottom_of, mid_left_of, mid_right_of};
pub use position::{Corner, Depth, Direction, Dimensions, HorizontalAlign, Margin, Padding,
                   Place, Point, Position, Positionable, Sizeable, VerticalAlign};
pub use theme::{Align, Theme};
pub use ui::{GlyphCache, Ui, UiId, UserInput};
pub use widget::{Widget, WidgetId};
pub use widget::State as WidgetState;


pub use json_io::Error as JsonIoError;


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


// #[macro_export]
// macro_rules! widget_ids {
//     ($($ids:ident),*) => {
//         $(
//             new_widget_id!(0, $($ids,)*);
//         )*
//     };
// }
// 
// macro_rules! new_widget_id{
//     ($count:expr, ,) => {};
//     ($count:expr, $current_id:ident) => {
//         const $current_id: WidgetId = $count;
//     };
//     ($count:expr, $current_id:ident, $($rest:ident,)*) => {
//         new_widget_id!($count, $current_id);
//         new_widget_id!($count + 1, $($rest),*,);
//     };
// }

