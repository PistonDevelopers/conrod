
use color::{Color, black, white};
use position::{Position, HorizontalAlign, VerticalAlign};
use rustc_serialize::{json, Encodable, Decodable};
use std::borrow::ToOwned;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::str;

/// A serializable collection of widget styling defaults.
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Theme {
    /// A name for the theme used for identification.
    pub name: String,
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
    /// A default widget position.
    pub position: Position,
    /// A default horizontal alignment for widgets.
    pub h_align: HorizontalAlign,
    /// A default vertical alignment for widgets.
    pub v_align: VerticalAlign,
    //TODO: Add unique theme-ing for each widget.
    //i.e. maybe_slider: Option<SliderTheme>, etc
}

impl Theme {

    /// The default theme if not loading from file.
    pub fn default() -> Theme {
        Theme {
            name: "Demo Theme".to_string(),
            background_color: black(),
            shape_color: white(),
            frame_color: black(),
            frame_width: 1.0,
            label_color: black(),
            font_size_large: 26,
            font_size_medium: 18,
            font_size_small: 12,
            position: Position::default(),
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Top,
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

