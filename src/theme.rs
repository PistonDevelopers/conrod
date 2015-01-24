
use color::Color;
use rustc_serialize::{
    json,
    Encodable,
    Decodable,
};
use std::error::Error;
use std::io::File;
use std::str;
use std::borrow::ToOwned;
use ui_context::UiContext;

/// A data holder for style-related data.
#[derive(Show, Clone, RustcEncodable, RustcDecodable)]
pub struct Theme {
    pub name: String,
    pub background_color: Color,
    pub shape_color: Color,
    pub frame_color: Color,
    pub frame_width: f64,
    pub label_color: Color,
    pub font_size_large: u32,
    pub font_size_medium: u32,
    pub font_size_small: u32,
    //TODO: Add unique theme-ing for each widget.
    //i.e. maybe_slider: Option<SliderTheme>, etc
}

impl Theme {

    /// The default theme if not loading from file.
    pub fn default() -> Theme {
        Theme {
            name: "Demo Theme".to_string(),
            background_color: Color::new(0.0, 0.0, 0.0, 1.0),
            shape_color: Color::new(1.0, 1.0, 1.0, 1.0),
            frame_color: Color::new(0.0, 0.0, 0.0, 1.0),
            frame_width: 1.0,
            label_color: Color::new(0.0, 0.0, 0.0, 1.0),
            font_size_large: 32,
            font_size_medium: 24,
            font_size_small: 18,
        }
    }

    /// Load a theme from file.
    pub fn load(path: &str) -> Result<Theme, String> {
        let contents = match File::open(&Path::new(path)).read_to_end() {
            Ok(buf) => Ok(buf),
            Err(e) => Err(format!("Failed to load Theme correctly: {}", e)),
        };
        let contents_str = contents.ok();
        let json_object = contents_str.and_then(|s| json::Json::from_str(str::from_utf8(s.as_slice()).unwrap()).ok());
        let mut decoder = json_object.map(|j| json::Decoder::new(j));
        let theme = match decoder {
            Some(ref mut d) => match Decodable::decode(d) {
                Ok(lib) => Ok(lib),
                Err(e) => Err(format!("Failed to load Theme correctly: {}", e.description())),
            },
            None => Err(String::from_str("Failed to load Theme correctly")),
        };
        theme
    }

    /// Save a theme to file.
    pub fn save(&self, path: &str) -> Result<(), String> {
        let json_string = match json::encode(self) {
                Ok(x) => x,
                Err(e) => return Err(e.description().to_owned())
            };
        let mut file = File::create(&Path::new(path));
        match file.write(json_string.as_bytes()) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Theme failed to save correctly: {}", e)),
        }
    }

}


/// A trait to make it easier to generically access the UIC on different widget contexts.
pub trait Themeable {
    /// Return a reference to the UiContext.
    fn get_theme(&self) -> &UiContext;
    /// Return a reference to the UiContext.
    fn get_theme_mut(&mut self) -> &mut UiContext;
}

