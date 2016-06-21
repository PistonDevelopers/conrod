//!
//! # Conrod
//!
//! An easy-to-use, immediate-mode, 2D GUI library featuring a range of useful widgets.
//!
//! If you are new to Conrod, we recommend checking out [The Guide](./guide/index.html).

#![deny(missing_copy_implementations)]
#![warn(missing_docs)]

extern crate daggy;
extern crate num;

extern crate graphics as piston_graphics;
extern crate input as piston_input;


pub use widget::primitive::line::Line;
pub use widget::primitive::image::Image;
pub use widget::primitive::point_path::PointPath;
pub use widget::primitive::shape::circle::Circle;
pub use widget::primitive::shape::framed_rectangle::FramedRectangle;
pub use widget::primitive::shape::polygon::Polygon;
pub use widget::primitive::shape::oval::Oval;
pub use widget::primitive::shape::rectangle::Rectangle;
pub use widget::primitive::text::{Text, Wrap as TextWrap};

pub use widget::button::Button;
pub use widget::canvas::Canvas;
pub use widget::drop_down_list::DropDownList;
pub use widget::envelope_editor::EnvelopeEditor;
pub use widget::envelope_editor::EnvelopePoint;
pub use widget::matrix::Matrix as WidgetMatrix;
pub use widget::number_dialer::NumberDialer;
pub use widget::plot_path::PlotPath;
pub use widget::range_slider::{RangeSlider, Edge as RangeSliderEdge};
pub use widget::scrollbar::Scrollbar;
pub use widget::slider::Slider;
pub use widget::tabs::Tabs;
pub use widget::text_box::TextBox;
pub use widget::text_edit::TextEdit;
pub use widget::title_bar::TitleBar;
pub use widget::toggle::Toggle;
pub use widget::xy_pad::XYPad;


pub use widget::primitive::line::Style as LineStyle;
pub use widget::primitive::image::Style as ImageStyle;
pub use widget::primitive::shape::Style as ShapeStyle;
pub use widget::primitive::shape::framed_rectangle::Style as FramedRectangleStyle;
pub use widget::primitive::text::Style as TextStyle;

pub use widget::button::Style as ButtonStyle;
pub use widget::canvas::Style as CanvasStyle;
pub use widget::drop_down_list::Style as DropDownListStyle;
pub use widget::envelope_editor::Style as EnvelopeEditorStyle;
pub use widget::number_dialer::Style as NumberDialerStyle;
pub use widget::plot_path::Style as PlotPathStyle;
pub use widget::range_slider::Style as RangeSliderStyle;
pub use widget::scrollbar::Style as ScrollbarStyle;
pub use widget::slider::Style as SliderStyle;
pub use widget::tabs::Style as TabsStyle;
pub use widget::text_box::Style as TextBoxStyle;
pub use widget::text_edit::Style as TextEditStyle;
pub use widget::title_bar::Style as TitleBarStyle;
pub use widget::toggle::Style as ToggleStyle;
pub use widget::xy_pad::Style as XYPadStyle;


pub use backend::{Backend, CharacterCache, Graphics};
pub use background::Background;
pub use color::{Color, Colorable};
pub use frame::{Framing, Frameable};
pub use glyph_cache::GlyphCache;
pub use graph::NodeIndex;
pub use label::{FontSize, Labelable};
pub use position::{Align, Axis, Corner, Depth, Direction, Dimension, Dimensions, Edge, Margin,
                   Padding, Place, Point, Position, Positionable, Range, Rect, Scalar, Sizeable};
//pub use position::Matrix as PositionMatrix;
pub use theme::Theme;
pub use ui::{Ui, UiCell};
pub use widget::{default_x_dimension, default_y_dimension};
pub use widget::scroll;
pub use widget::{CommonBuilder, CommonState, CommonStyle, Floating, IndexSlot, MaybeParent,
                 UpdateArgs, Widget};
pub use widget::{KidArea, KidAreaArgs};
pub use widget::CommonState as WidgetCommonState;
pub use widget::Id as WidgetId;
pub use widget::Index as WidgetIndex;
pub use widget::Kind as WidgetKind;
pub use widget::State as WidgetState;


pub mod backend;
mod background;
pub mod color;
pub mod event;
mod frame;
pub mod glyph_cache;
pub mod graph;
pub mod guide;
pub mod input;
mod label;
mod position;
pub mod text;
pub mod theme;
mod ui;
pub mod utils;
mod widget;

#[cfg(test)]
mod tests;




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
