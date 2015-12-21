//!
//! Types a functionality for handling Canvas and Widget theming.
//!

use Scalar;
use color::{Color, black, white};
use position::{Align, Direction, Padding, Position};
use std::any::Any;
use std::collections::HashMap;
use widget;


/// A serializable collection of canvas and widget styling defaults.
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
    /// A default color for widget frames.
    pub frame_color: Color,
    /// A default width for widget frames.
    pub frame_width: Scalar,
    /// A default color for widget labels.
    pub label_color: Color,
    /// A default "large" font size.
    pub font_size_large: u32,
    /// A default "medium" font size.
    pub font_size_medium: u32,
    /// A default "small" font size.
    pub font_size_small: u32,
    /// Optional style defaults for a Scrollbar.
    pub maybe_scrollbar: Option<widget::scroll::Style>,
    /// Unique styling for each widget, index-able by the **Widget::kind**.
    pub widget_styling: HashMap<&'static str, WidgetDefault>,
}

/// The defaults for a specific widget.
pub struct WidgetDefault {
    /// The unique style of a widget.
    pub style: Box<Any>,
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
    pub fn new(style: Box<Any>) -> WidgetDefault {
        WidgetDefault {
            style: style,
            common: widget::CommonStyle::new(),
        }
    }
}


impl Theme {

    /// The default theme if not loading from file.
    pub fn default() -> Theme {
        Theme {
            name: "Demo Theme".to_string(),
            padding: Padding::none(),
            x_position: Position::Align(Align::Start, None),
            y_position: Position::Direction(Direction::Backwards, 20.0, None),
            background_color: black(),
            shape_color: white(),
            frame_color: black(),
            frame_width: 1.0,
            label_color: black(),
            font_size_large: 26,
            font_size_medium: 18,
            font_size_small: 12,
            maybe_scrollbar: None,
            widget_styling: HashMap::new(),
        }
    }

    /// Retrieve the unique default styling for a widget.
    ///
    /// Attempts to cast the `Box<WidgetStyle>` to the **Widget**'s unique style **T**.
    pub fn widget_style<T>(&self, kind: &'static str) -> Option<UniqueDefault<T>>
        where T: widget::Style,
    {
        self.widget_styling.get(kind).and_then(|boxed_default| {
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

