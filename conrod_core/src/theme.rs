//!
//! Types a functionality for handling Canvas and Widget theming.
//!

use Scalar;
use color::{Color, BLACK, WHITE};
use position::{Align, Direction, Padding, Position, Relative};
use fnv;
use std;
use std::any::Any;
use text;
use widget;

/// `std::collections::HashMap` with `fnv::FnvHasher` for unique styling
/// of each widget, index-able by the **Widget::kind**.
pub type StyleMap = fnv::FnvHashMap<std::any::TypeId, WidgetDefault>;


/// A serializable collection of canvas and widget styling defaults.
#[derive(Debug)]
pub struct Theme {
    /// A name for the theme used for identification.
    pub name: String,
    /// Padding for Canvas layout and positioning.
    pub padding: Padding,
    /// A default widget position along the *x* axis.
    pub x_position: Position,
    /// A default widget position along the *y* axis.
    pub y_position: Position,
    /// A default background for the theme.
    pub background_color: Color,
    /// A default color for widget shapes.
    pub shape_color: Color,
    /// A default color for widget borders.
    pub border_color: Color,
    /// A default width for widget borders.
    pub border_width: Scalar,
    /// A default color for widget labels.
    pub label_color: Color,
    /// The `Id` of the default font used for text widgets when one is not specified.
    pub font_id: Option<text::font::Id>,
    /// A default "large" font size.
    pub font_size_large: u32,
    /// A default "medium" font size.
    pub font_size_medium: u32,
    /// A default "small" font size.
    pub font_size_small: u32,
    /// `StyleMap` for unique styling
    /// of each widget, index-able by the **Widget::kind**.
    pub widget_styling: StyleMap,
    /// Mouse Drag distance threshold determines the minimum distance from the mouse-down point
    /// that the mouse must move before starting a drag operation.
    pub mouse_drag_threshold: Scalar,
    /// Once the `Duration` that separates two consecutive `Click`s is greater than this value, a
    /// `DoubleClick` event will no longer be generated.
    pub double_click_threshold: std::time::Duration,
}

/// The defaults for a specific widget.
#[derive(Debug)]
pub struct WidgetDefault {
    /// The unique style of a widget.
    pub style: Box<Any + Send>,
    /// The attributes commonly shared between widgets.
    pub common: widget::CommonStyle,
}

/// A **WidgetDefault** downcast to a **Widget**'s unique **Style** type.
#[derive(Copy, Clone, Debug)]
pub struct UniqueDefault<'a, T: 'a> {
    /// The unique style for the widget.
    pub style: &'a T,
    /// Attributes that are common to all widgets.
    pub common: &'a widget::CommonStyle,
}

impl WidgetDefault {
    /// Constructor for a WidgetDefault.
    pub fn new(style: Box<Any + Send>) -> WidgetDefault {
        WidgetDefault {
            style: style,
            common: widget::CommonStyle::default(),
        }
    }
}

impl Theme {

    /// The default theme if not loading from file.
    pub fn default() -> Theme {
        Theme {
            name: "Demo Theme".to_string(),
            padding: Padding::none(),
            x_position: Position::Relative(Relative::Align(Align::Start), None),
            y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
            background_color: BLACK,
            shape_color: WHITE,
            border_color: BLACK,
            border_width: 1.0,
            label_color: BLACK,
            font_id: None,
            font_size_large: 26,
            font_size_medium: 18,
            font_size_small: 12,
            widget_styling: fnv::FnvHashMap::default(),
            mouse_drag_threshold: 0.0,
            double_click_threshold: std::time::Duration::from_millis(500),
        }
    }

    /// Retrieve the unique default styling for a widget.
    ///
    /// Attempts to cast the `Box<WidgetStyle>` to the **Widget**'s unique associated style **T**.
    pub fn widget_style<T>(&self) -> Option<UniqueDefault<T>>
        where T: widget::Style,
    {
        let style_id = std::any::TypeId::of::<T>();
        self.widget_styling.get(&style_id).and_then(|boxed_default| {
            boxed_default.style.downcast_ref().map(|style| {
                let common = &boxed_default.common;
                UniqueDefault {
                    style: style,
                    common: common,
                }
            })
        })
    }

}
