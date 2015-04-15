
use color::{Color, black, white};
use rustc_serialize::{
    json,
    Encodable,
    Decodable,
};
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::str;
use std::borrow::ToOwned;
use ui::Ui;

/// A data holder for style-related data.
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
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
            background_color: black(),
            shape_color: white(),
            frame_color: black(),
            frame_width: 1.0,
            label_color: black(),
            font_size_large: 26,
            font_size_medium: 18,
            font_size_small: 12,
        }
    }

    /// Load a theme from file.
    pub fn load(path: &str) -> Result<Theme, String> {
        let mut file = match File::open(&Path::new(path)) {
            Ok(file) => file,
            Err(e) => return Err(format!("Failed to open file for Theme: {}", Error::description(&e))),
        };
        let mut contents = Vec::new();
        if let Err(e) = ::std::io::Read::read_to_end(&mut file, &mut contents) {
            return Err(format!("Failed to load Theme correctly: {}", Error::description(&e)));
        }
        let json_object = match json::Json::from_str(str::from_utf8(&contents[..]).unwrap()) {
            Ok(json_object) => json_object,
            Err(e) => return Err(format!("Failed to construct json_object from str: {}", Error::description(&e))),
        };
        let mut decoder = json::Decoder::new(json_object);
        let theme = match Decodable::decode(&mut decoder) {
            Ok(theme) => Ok(theme),
            Err(e) => Err(format!("Failed to construct Theme from json decoder: {}", Error::description(&e))),
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
            Err(e) => return Err(format!("Failed to create a File at the given path: {}", Error::description(&e)))
        };
        match ::std::io::Write::write_all(&mut file, json_string.as_bytes()) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Theme failed to save correctly: {}", Error::description(&e))),
        }
    }

}


/// A trait to make it easier to generically access the UIC on different widget contexts.
pub trait Themeable<C> {
    /// Return a reference to the UiContext.
    fn get_theme(&self) -> &Ui<C>;
    /// Return a reference to the UiContext.
    fn get_theme_mut(&mut self) -> &mut Ui<C>;
}
