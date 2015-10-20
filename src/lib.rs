//!
//! # Conrod
//!
//! An easy-to-use, immediate-mode, 2D GUI library featuring a range of useful widgets.
//!

#![deny(missing_copy_implementations)]
#![warn(missing_docs)]

#[macro_use] extern crate bitflags;
extern crate clock_ticks;
extern crate daggy;
extern crate elmesque;
extern crate graphics;
extern crate json_io;
extern crate num;
extern crate input;
extern crate rand;
extern crate rustc_serialize;
extern crate vecmath;


pub use widget::button::Button;
pub use widget::canvas::Canvas;
pub use widget::drop_down_list::DropDownList;
pub use widget::envelope_editor::EnvelopeEditor;
pub use widget::envelope_editor::EnvelopePoint;
pub use widget::label::Label;
pub use widget::matrix::Matrix as WidgetMatrix;
pub use widget::number_dialer::NumberDialer;
pub use widget::slider::Slider;
pub use widget::split::Split;
pub use widget::tabs::Tabs;
pub use widget::text_box::TextBox;
pub use widget::toggle::Toggle;
pub use widget::xy_pad::XYPad;

pub use widget::button::Style as ButtonStyle;
pub use widget::canvas::Style as CanvasStyle;
pub use widget::drop_down_list::Style as DropDownListStyle;
pub use widget::envelope_editor::Style as EnvelopeEditorStyle;
pub use widget::label::Style as LabelStyle;
pub use widget::number_dialer::Style as NumberDialerStyle;
pub use widget::slider::Style as SliderStyle;
pub use widget::tabs::Style as TabsStyle;
pub use widget::text_box::Style as TextBoxStyle;
pub use widget::toggle::Style as ToggleStyle;
pub use widget::xy_pad::Style as XYPadStyle;


pub use background::Background;
pub use elmesque::{color, Element};
pub use elmesque::color::{Color, Colorable};
pub use frame::{Framing, Frameable};
pub use graph::NodeIndex;
pub use graphics::character::CharacterCache;
pub use graphics::math::Scalar;
pub use label::{FontSize, Labelable};
pub use mouse::Mouse;
pub use mouse::ButtonState as MouseButtonState;
pub use mouse::ButtonPosition as MouseButtonPosition;
pub use mouse::Scroll as MouseScroll;
pub use position::{align_left_of, align_right_of, align_bottom_of, align_top_of};
pub use position::{middle_of, top_left_of, top_right_of, bottom_left_of, bottom_right_of,
                   mid_top_of, mid_bottom_of, mid_left_of, mid_right_of};
pub use position::{Corner, Depth, Direction, Dimensions, Horizontal, HorizontalAlign, Margin,
                   Padding, Place, Point, Position, Positionable, Range, Rect, Sizeable,
                   Vertical, VerticalAlign};
pub use position::Matrix as PositionMatrix;
pub use theme::{Align, Theme};
pub use ui::{GlyphCache, Ui, UserInput};
pub use widget::{drag, scroll};
pub use widget::{CommonBuilder, CommonState, DrawArgs, Floating, MaybeParent, UiCell, UpdateArgs,
                 Widget};
pub use widget::{KidArea, KidAreaArgs};
pub use widget::CommonState as WidgetCommonState;
pub use widget::Id as WidgetId;
pub use widget::Index as WidgetIndex;
pub use widget::State as WidgetState;


pub use json_io::Error as JsonIoError;


mod background;
mod frame;
mod graph;
mod label;
mod mouse;
mod position;
pub mod theme;
mod ui;
pub mod utils;
mod widget;




/// Generate a list of unique IDs given a list of identifiers.
///
/// This is the recommended way of generating `WidgetId`s as it greatly lessens the chances of
/// making errors when adding or removing widget ids.
///
/// Each Widget must have its own unique identifier so that the `Ui` can keep track of its state 
/// between updates.
///
/// To make this easier, we provide the `widget_ids` macro, which generates a unique `WidgetId` for 
/// each identifier given in the list.
///
/// The `with n` syntax reserves `n` number of `WidgetId`s for that identifier rather than just one.
///
/// This is often useful in the case that you need to set multiple Widgets in a loop or when using
/// the `widget::Matrix`.
///
/// Note: Make sure when that you remember to `#[macro_use]` if you want to use this macro - i.e.
///
/// `#[macro_use] extern crate conrod;`
///
/// Also, if your list has a large number of identifiers (~64 or more) you may find this macro
/// hitting rustc's recursion limit (this will show as a compile error). To fix this you can try
/// adding the following to your crate root.
///
/// `#![recursion_limit="512"]`
///
/// This will raise the recursion limit from the default (~64) to 512. You should be able to set it
/// to a higher number if you find it necessary.
///
#[macro_export]
macro_rules! widget_ids {

    // Handle the first ID.
    ( $widget_id:ident , $($rest:tt)* ) => (
        const $widget_id: $crate::WidgetId = $crate::WidgetId(0);
        widget_ids!($widget_id.0 => $($rest)*);
    );

    // Handle the first ID with some given step between it and the next ID.
    ( $widget_id:ident with $step:expr , $($rest:tt)* ) => (
        const $widget_id: $crate::WidgetId = $crate::WidgetId(0);
        widget_ids!($widget_id.0 + $step => $($rest)*);
    );

    // Handle some consecutive ID.
    ( $prev_id:expr => $widget_id:ident , $($rest:tt)* ) => (
        const $widget_id: $crate::WidgetId = $crate::WidgetId($prev_id + 1);
        widget_ids!($widget_id.0 => $($rest)*);
    );

    // Handle some consecutive ID with some given step between it and the next ID.
    ( $prev_id:expr => $widget_id:ident with $step:expr , $($rest:tt)* ) => (
        const $widget_id: $crate::WidgetId = $crate::WidgetId($prev_id + 1);
        widget_ids!($widget_id.0 + $step => $($rest)*);
    );


    ///// End cases. /////


    // Handle the final ID.
    () => ();

    // Handle the final ID.
    ( $prev_id:expr => ) => ();


    ///// Handle end cases that don't have a trailing comma. /////


    // Handle a single ID without a trailing comma.
    ( $widget_id:ident ) => (
        const $widget_id: $crate::WidgetId = $crate::WidgetId(0);
    );

    // Handle a single ID with some given step without a trailing comma.
    ( $widget_id:ident with $step:expr ) => (
        const $widget_id: $crate::WidgetId = $crate::WidgetId(0);
    );

    // Handle the last ID without a trailing comma.
    ( $prev_id:expr => $widget_id:ident ) => (
        const $widget_id: $crate::WidgetId = $crate::WidgetId($prev_id + 1);
    );

    // Handle the last ID with some given step without a trailing comma.
    ( $prev_id:expr => $widget_id:ident with $step:expr ) => (
        const $widget_id: $crate::WidgetId = $crate::WidgetId($prev_id + 1);
    );

}


#[test]
fn test() {
    widget_ids! {
        A,
        B with 64,
        C with 32,
        D,
        E with 8,
    }
    assert_eq!(A, WidgetId(0));
    assert_eq!(B, WidgetId(1));
    assert_eq!(C, WidgetId(66));
    assert_eq!(D, WidgetId(99));
    assert_eq!(E, WidgetId(100));
}
