//!
//! Types a functionality for handling Canvas and Widget theming.
//!

use canvas;
use color::{Color, black, white};
use json_io;
use position::{Margin, Padding, Position, HorizontalAlign, VerticalAlign};
use rustc_serialize::Encodable;
use std::error::Error;
use std::path::Path;
use widget;


/// A serializable collection of canvas and widget styling defaults.
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
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
    pub frame_width: f64,
    /// A default color for widget labels.
    pub label_color: Color,
    /// A default "large" font size.
    pub font_size_large: u32,
    /// A default "medium" font size.
    pub font_size_medium: u32,
    /// A default "small" font size.
    pub font_size_small: u32,
    /// Optional style defaults for a Canvas split.
    pub maybe_canvas_split: Option<canvas::split::Style>,
    /// Optional style defaults for a Floating Canvas.
    pub maybe_canvas_floating: Option<canvas::floating::Style>,
    /// Optional style defaults for a Button widget.
    pub maybe_button: Option<widget::button::Style>,
    /// Optional style defaults for a DropDownList.
    pub maybe_drop_down_list: Option<widget::drop_down_list::Style>,
    /// Optional style defaults for an EnvelopeEditor.
    pub maybe_envelope_editor: Option<widget::envelope_editor::Style>,
    /// Optional style defaults for a NumberDialer.
    pub maybe_number_dialer: Option<widget::number_dialer::Style>,
    /// Optional style defaults for a Slider.
    pub maybe_slider: Option<widget::slider::Style>,
    /// Optional style defaults for a TextBox.
    pub maybe_text_box: Option<widget::text_box::Style>,
    /// Optional style defaults for a Toggle.
    pub maybe_toggle: Option<widget::toggle::Style>,
    /// Optional style defaults for an XYPad.
    pub maybe_xy_pad: Option<widget::xy_pad::Style>,
}

/// The alignment of an element's dimensions with another's.
#[derive(Copy, Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Align {
    /// Positioning relative to an elements width and position on the x axis.
    pub horizontal: HorizontalAlign,
    /// Positioning relative to an elements height and position on the y axis.
    pub vertical: VerticalAlign,
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
                horizontal: HorizontalAlign::Left,
                vertical: VerticalAlign::Top,
            },
            background_color: black(),
            shape_color: white(),
            frame_color: black(),
            frame_width: 1.0,
            label_color: black(),
            font_size_large: 26,
            font_size_medium: 18,
            font_size_small: 12,
            maybe_canvas_split: None,
            maybe_canvas_floating: None,
            maybe_button: None,
            maybe_drop_down_list: None,
            maybe_envelope_editor: None,
            maybe_number_dialer: None,
            maybe_slider: None,
            maybe_text_box: None,
            maybe_toggle: None,
            maybe_xy_pad: None,
        }
    }

    /// Load a theme from file.
    pub fn load(path: &Path) -> Result<Theme, json_io::Error> {
        json_io::load(path)
    }

    /// Save a theme to file.
    pub fn save(&self, path: &Path) -> Result<(), json_io::Error> {
        json_io::save(path, self)
    }

}

