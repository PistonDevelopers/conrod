
use freetype;
use label::FontSize;
use opengl_graphics::{
    Texture,
};
use piston::{
    AssetStore,
};
use std::collections::HashMap;

/// Struct used to hold rendered character data.
pub struct Character {
    pub glyph: freetype::Glyph,
    pub bitmap_glyph: freetype::BitmapGlyph,
    pub texture: Texture,
}

/// A struct used for caching rendered font.
pub struct GlyphCache {
    pub face: freetype::Face,
    data: HashMap<FontSize, HashMap<char, Character>>,
}

impl GlyphCache {

    /// Constructor for a GlyphCache.
    pub fn new(font_file: &str) -> GlyphCache {
        let freetype = freetype::Library::init().unwrap();
        let asset_store = AssetStore::from_folder("../assets");
        let font = asset_store.path(font_file).unwrap();
        let face = freetype.new_face(font.as_str().unwrap(), 0).unwrap();
        GlyphCache {
            face: face,
            data: HashMap::new(),
        }
    }

    /// Return a reference to a `Character`. If there is not yet a `Character` for
    /// the given `FontSize` and `char`, load the `Character`.
    pub fn get_character(&mut self, size: FontSize, ch: char) -> &Character {
        match self.data.find_or_insert(size, HashMap::new()).contains_key(&ch) {
            true => self.data.get(&size).get(&ch),
            false => { self.load_character(size, ch); self.data.get(&size).get(&ch) }
        }
    }

    /// Load a `Character` from a given `FontSize` and `char`.
    fn load_character(&mut self, size: FontSize, ch: char) {
        self.face.set_pixel_sizes(0, size).unwrap();
        self.face.load_char(ch as u64, freetype::face::Default).unwrap();
        let glyph = self.face.glyph().get_glyph().unwrap();
        let bitmap_glyph = glyph.to_bitmap(freetype::render_mode::Normal, None).unwrap();
        let bitmap = bitmap_glyph.bitmap();
        let texture = Texture::from_memory_alpha(bitmap.buffer(),
                                                 bitmap.width() as u32,
                                                 bitmap.rows() as u32).unwrap();
        self.data.get_mut(&size).insert(ch, Character {
            glyph: glyph,
            bitmap_glyph: bitmap_glyph,
            texture: texture,
        });
    }

}

