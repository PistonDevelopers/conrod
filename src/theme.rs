
/// CURRENTLY UNUSABLE!

/// A data holder for style-related data.
#[deriving(Show, Clone, Encodable, Decodable)]
pub struct Theme {
    name: String,
    shape_color: Color,
    frame_color: Color,
    frame_width: f64,
    label_color: Color,
    font_size_large: u32,
    font_size_medium: u32,
    font_size_small: u32,
}

/// A library for storing multiple themes.
#[deriving(Show, Clone, Encodable, Decodable)]
pub struct Library {
    themes: Vec<Theme>,
}

impl Library {

    /// Load a theme from file.
    pub fn load(name: &str, path: &str) -> Theme {
        let contents = match File::open(&Path::new(path)).read_to_end() {
            Ok(buf) => buf,
            Err(e) => fail!("Failed to load Library correctly: {}", e),
        };
        let contents_str = match str::from_utf8(contents.as_slice()) {
            Some(string) => string,
            Err(e) => fail!("Failed to load Library correctly: {}", e),
        };
        let json_object = json::from_str(contents_str);
        let mut decoder = json::Decoder::new(json_object.unwrap());
        let theme: Theme = match Decodable::decode(&mut decoder) {
            Ok(lib) => lib,
            Err(e) => fail!("Failed to load Library correctly: {}", e),
        };
        theme
    }

    /// Save a theme to file.
    pub fn save(&self, path: &str) {
        let json_string = json::Encoder::buffer_encode(self);
        let mut file = File::create(&Path::new(path));
        match file.write(json_string.as_slice()) {
            Ok(()) => (),
            Err(e) => fail!("Library failed to save correctly: {}", e),
        }
    }

}

