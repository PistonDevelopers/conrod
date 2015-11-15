//!
//! Types a functionality for handling Canvas and Widget theming.
//!

use Scalar;
use color::{Color, black, white};
//use json_io;
use position::{Margin, Padding, Position, Horizontal, HorizontalAlign, Vertical, VerticalAlign};
use rustc_serialize::Encodable;
use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
// use std::fmt::Debug;
// use std::path::Path;
use widget;


/// A serializable collection of canvas and widget styling defaults.
// #[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Theme {
    /// A name for the theme used for identification.
    pub name: String,
    /// Padding for Canvas layout and positioning.
    pub padding: Padding,
    /// Margin for Canvas layout and positioning.
    pub margin: Margin,
    /// A default widget position.
    pub position: Position,
    /// A default alignment for widgets.
    pub align: Align,
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

    // /// Optional style defaults for a Button widget.
    // pub maybe_button: Option<WidgetDefault<widget::button::Style>>,
    // /// Optional style defaults for a Canvas widget.
    // pub maybe_canvas: Option<WidgetDefault<widget::canvas::Style>>,
    // /// Optional style defaults for a DropDownList.
    // pub maybe_drop_down_list: Option<WidgetDefault<widget::drop_down_list::Style>>,
    // /// Optional style defaults for an EnvelopeEditor.
    // pub maybe_envelope_editor: Option<WidgetDefault<widget::envelope_editor::Style>>,
    // /// Optional style defaults for a Line.
    // pub maybe_line: Option<WidgetDefault<widget::primitive::line::Style>>,
    // /// Optional style defaults for a Matrix.
    // pub maybe_matrix: Option<WidgetDefault<widget::matrix::Style>>,
    // /// Optional style defaults for a NumberDialer.
    // pub maybe_number_dialer: Option<WidgetDefault<widget::number_dialer::Style>>,
    // /// Optional style defaults for a PointPath.
    // pub maybe_point_path: Option<WidgetDefault<widget::primitive::point_path::Style>>,
    // /// Optional style defaults for a Slider.
    // pub maybe_slider: Option<WidgetDefault<widget::slider::Style>>,
    // /// Optional style defaults for a Tabs widget.
    // pub maybe_tabs: Option<WidgetDefault<widget::tabs::Style>>,
    // /// Optional style defaults for a TextBox.
    // pub maybe_text_box: Option<WidgetDefault<widget::text_box::Style>>,
    // /// Optional style defaults for a Toggle.
    // pub maybe_toggle: Option<WidgetDefault<widget::toggle::Style>>,
    // /// Optional style defaults for an XYPad.
    // pub maybe_xy_pad: Option<WidgetDefault<widget::xy_pad::Style>>,
}

/// The alignment of an element's dimensions with another's.
#[derive(Copy, Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Align {
    /// Positioning relative to an elements width and position on the x axis.
    pub horizontal: HorizontalAlign,
    /// Positioning relative to an elements height and position on the y axis.
    pub vertical: VerticalAlign,
}

/// The defaults for a specific widget.
pub struct WidgetDefault {
    /// The unique style of a widget.
    pub style: Box<Any>,
    /// The attributes commonly shared between widgets.
    pub common: widget::CommonBuilder,
}

/// A **WidgetDefault** downcast to a **Widget**'s unique **Style** type.
#[derive(Copy, Clone, Debug)]
pub struct UniqueDefault<'a, T: 'a> {
    /// The unique style for the widget.
    pub style: &'a T,
    /// Attributes that are common to between all widgets.
    pub common: &'a widget::CommonBuilder,
}

impl WidgetDefault {
    /// Constructor for a WidgetDefault.
    pub fn new(style: Box<Any>) -> WidgetDefault {
        WidgetDefault {
            style: style,
            common: widget::CommonBuilder::new(),
        }
    }
}


impl Theme {

    /// The default theme if not loading from file.
    pub fn default() -> Theme {
        Theme {
            name: "Demo Theme".to_string(),
            padding: Padding {
                top: 0.0,
                bottom: 0.0,
                left: 0.0,
                right: 0.0,
            },
            margin: Margin {
                top: 0.0,
                bottom: 0.0,
                left: 0.0,
                right: 0.0,
            },
            position: Position::default(),
            align: Align {
                horizontal: HorizontalAlign(Horizontal::Left, None),
                vertical: VerticalAlign(Vertical::Top, None),
            },
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

            // maybe_button: None,
            // maybe_canvas: None,
            // maybe_drop_down_list: None,
            // maybe_envelope_editor: None,
            // maybe_line: None,
            // maybe_matrix: None,
            // maybe_number_dialer: None,
            // maybe_point_path: None,
            // maybe_slider: None,
            // maybe_tabs: None,
            // maybe_text_box: None,
            // maybe_toggle: None,
            // maybe_xy_pad: None,
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

    // /// Load a theme from file.
    // pub fn load(path: &Path) -> Result<Theme, json_io::Error> {
    //     json_io::load(path)
    // }

    // /// Save a theme to file.
    // pub fn save(&self, path: &Path) -> Result<(), json_io::Error> {
    //     json_io::save(path, self)
    // }

}

