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
extern crate input as piston_input;
extern crate rusttype;


pub use color::{Color, Colorable};
pub use border::{Bordering, Borderable};
pub use graph::NodeIndex;
pub use label::{FontSize, Labelable};
pub use position::{Align, Axis, Corner, Depth, Direction, Dimension, Dimensions, Edge, Margin,
                   Padding, Place, Point, Position, Positionable, Range, Rect, Scalar, Sizeable};
pub use theme::Theme;
pub use ui::{Ui, UiCell, UiBuilder};
pub use widget::{scroll, Widget};


pub mod backend;
mod border;
pub mod color;
pub mod event;
pub mod graph;
pub mod guide;
pub mod image;
pub mod input;
mod label;
mod position;
pub mod render;
pub mod text;
pub mod theme;
mod ui;
pub mod utils;
pub mod widget;

#[cfg(test)] mod tests;


/// Generate a list of unique IDs given a list of identifiers.
///
/// This is the recommended way of generating `widget::Id`s as it greatly lessens the chances of
/// making errors when adding or removing widget ids.
///
/// Each Widget must have its own unique identifier so that the `Ui` can keep track of its state
/// between updates.
///
/// To make this easier, we provide the `widget_ids` macro, which generates a unique `widget::Id`
/// for each identifier given in the list.
///
/// The `with n` syntax reserves `n` number of `widget::Id`s for that identifier rather than just
/// one.
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
#[macro_export]
macro_rules! widget_ids {

    // Handle the first ID.
    ( $widget_id:ident , $($rest:tt)* ) => (
        const $widget_id: $crate::widget::Id = $crate::widget::Id(0);
        widget_ids!($widget_id.0 => $($rest)*);
    );

    // Handle the first ID with some given step between it and the next ID.
    ( $widget_id:ident with $step:expr , $($rest:tt)* ) => (
        const $widget_id: $crate::widget::Id = $crate::widget::Id(0);
        widget_ids!($widget_id.0 + $step => $($rest)*);
    );

    // Handle some consecutive ID.
    ( $prev_id:expr => $widget_id:ident , $($rest:tt)* ) => (
        const $widget_id: $crate::widget::Id = $crate::widget::Id($prev_id + 1);
        widget_ids!($widget_id.0 => $($rest)*);
    );

    // Handle some consecutive ID with some given step between it and the next ID.
    ( $prev_id:expr => $widget_id:ident with $step:expr , $($rest:tt)* ) => (
        const $widget_id: $crate::widget::Id = $crate::widget::Id($prev_id + 1);
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
        const $widget_id: $crate::widget::Id = $crate::widget::Id(0);
    );

    // Handle a single ID with some given step without a trailing comma.
    ( $widget_id:ident with $step:expr ) => (
        const $widget_id: $crate::widget::Id = $crate::widget::Id(0);
    );

    // Handle the last ID without a trailing comma.
    ( $prev_id:expr => $widget_id:ident ) => (
        const $widget_id: $crate::widget::Id = $crate::widget::Id($prev_id + 1);
    );

    // Handle the last ID with some given step without a trailing comma.
    ( $prev_id:expr => $widget_id:ident with $step:expr ) => (
        const $widget_id: $crate::widget::Id = $crate::widget::Id($prev_id + 1);
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
    assert_eq!(A, widget::Id(0));
    assert_eq!(B, widget::Id(1));
    assert_eq!(C, widget::Id(66));
    assert_eq!(D, widget::Id(99));
    assert_eq!(E, widget::Id(100));
}
