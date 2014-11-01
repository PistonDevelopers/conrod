
use color::Color;
use serialize::{
    json,
    Encodable,
    Decodable
};
use std::io::File;
use std::str;
use ui_context::UiContext;

/// A data holder for style-related data.
#[deriving(Show, Clone, Encodable, Decodable)]
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
    pub fn load(path: &str) -> Theme {
        let contents = match File::open(&Path::new(path)).read_to_end() {
            Ok(buf) => buf,
            Err(e) => panic!("Failed to load Theme correctly: {}", e),
        };
        let contents_str = match str::from_utf8(contents.as_slice()) {
            Some(string) => string,
            None => panic!("Failed to load Theme!"),
        };
        let json_object = json::from_str(contents_str);
        let mut decoder = json::Decoder::new(json_object.unwrap());
        let theme: Theme = match Decodable::decode(&mut decoder) {
            Ok(lib) => lib,
            Err(e) => panic!("Failed to load Theme correctly: {}", e),
        };
        theme
    }

    /// Save a theme to file.
    pub fn save(&self, path: &str) {
        let json_string = json::Encoder::buffer_encode(self);
        let mut file = File::create(&Path::new(path));
        match file.write(json_string.as_slice()) {
            Ok(()) => (),
            Err(e) => panic!("Theme failed to save correctly: {}", e),
        }
    }

}


/// A trait to make it easier to generically access the UIC on different widget contexts.
pub trait Themeable<T> {
    /// Return a reference to the UiContext.
    fn get_theme(&self) -> &UiContext<T>;
    /// Return a reference to the UiContext.
    fn get_theme_mut(&mut self) -> &mut UiContext<T>;
}

