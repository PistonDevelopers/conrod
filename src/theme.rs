
use canvas;
use color::{Color, black, white};
use position::{Margin, Padding, Position, HorizontalAlign, VerticalAlign};
use rustc_serialize::{json, Encodable, Decodable};
use std::borrow::ToOwned;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::str;
use widget::{button};

/// A serializable collection of widget styling defaults.
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
    /// Specific defaults for a Button widget.
    pub maybe_button: Option<button::Style>,
}

/// The alignment of an element's dimensions with another's.
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
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
            maybe_button: None,
        }
    }

    /// Load a theme from file.
    pub fn load(path: &str) -> Result<Theme, String> {
        let mut file = match File::open(&Path::new(path)) {
            Ok(file) => file,
            Err(e) => return Err(format!("Failed to open file for Theme: {}",
                                         Error::description(&e))),
        };
        let mut contents = Vec::new();
        if let Err(e) = ::std::io::Read::read_to_end(&mut file, &mut contents) {
            return Err(format!("Failed to load Theme correctly: {}",
                               Error::description(&e)));
        }
        let json_object = match json::Json::from_str(str::from_utf8(&contents[..]).unwrap()) {
            Ok(json_object) => json_object,
            Err(e) => return Err(format!("Failed to construct json_object from str: {}",
                                         Error::description(&e))),
        };
        let mut decoder = json::Decoder::new(json_object);
        let theme = match Decodable::decode(&mut decoder) {
            Ok(theme) => Ok(theme),
            Err(e) => Err(format!("Failed to construct Theme from json decoder: {}",
                                  Error::description(&e))),
        };
        theme
    }

    /// Save a theme to file.
    pub fn save(&self, path: &str) -> Result<(), String> {
        let json_string = match json::encode(self) {
            Ok(x) => x,
            Err(e) => return Err(e.description().to_owned())
        };
        let mut file = match File::create(&Path::new(path)) {
            Ok(file) => file,
            Err(e) => return Err(format!("Failed to create a File at the given path: {}",
                                         Error::description(&e)))
        };
        match ::std::io::Write::write_all(&mut file, json_string.as_bytes()) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Theme failed to save correctly: {}", Error::description(&e))),
        }
    }

}

