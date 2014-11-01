
use freetype;
use label::FontSize;
use std::collections::HashMap;
use std::collections::hashmap::{Occupied, Vacant};

/// Struct used to hold rendered character data.
pub struct Character<T> {
    pub glyph: freetype::Glyph,
    pub bitmap_glyph: freetype::BitmapGlyph,
    pub texture: T,
}

pub type TextureConstructor<T> = |buffer: &[u8], width: u32, height: u32|: 'static -> T;

/// A struct used for caching rendered font.
pub struct GlyphCache<T> {
    pub face: freetype::Face,
    pub texture_constructor: Option<TextureConstructor<T>>,
    data: HashMap<FontSize, HashMap<char, Character<T>>>,
}

impl<T> GlyphCache<T> {

    /// Constructor for a GlyphCache.
    pub fn new(font: &Path) -> GlyphCache<T> {
        let freetype = freetype::Library::init().unwrap();
        let font_str = match font.as_str() {
            Some(font_str) => font_str,
            None => panic!("GlyphCache::new() : Failed to return `font.as_str()`."),
        };
        let face = match freetype.new_face(font_str, 0) {
            Ok(face) => face,
            Err(err) => panic!("GlyphCache::new() : {}", err),
        };
        GlyphCache {
            face: face,
            data: HashMap::new(),
            texture_constructor: None,
        }
    }

    /// Return a reference to a `Character`. If there is not yet a `Character` for
    /// the given `FontSize` and `char`, load the `Character`.
    pub fn get_character(&mut self, size: FontSize, ch: char) -> &Character<T> {        
        match {
            match self.data.entry(size) {
                Vacant(entry) => entry.set(HashMap::new()),
                Occupied(entry) => entry.into_mut(),
            }
        }.contains_key(&ch) {
            true => &self.data[size][ch],
            false => { self.load_character(size, ch); &self.data[size][ch] }
        }
    }
    
    /// Load a `Character` from a given `FontSize` and `char`.
    fn load_character(&mut self, size: FontSize, ch: char) {
        self.face.set_pixel_sizes(0, size).unwrap();
        self.face.load_char(ch as u64, freetype::face::DEFAULT).unwrap();
        let glyph = self.face.glyph().get_glyph().unwrap();
        let bitmap_glyph = glyph.to_bitmap(freetype::render_mode::Normal, None).unwrap();
        let bitmap = bitmap_glyph.bitmap();
        let texture = (*self.texture_constructor.as_mut().unwrap())(
            bitmap.buffer(), bitmap.width() as u32, bitmap.rows() as u32);
        self.data[size].insert(ch, Character {
            glyph: glyph,
            bitmap_glyph: bitmap_glyph,
            texture: texture,
        });
    }
}


