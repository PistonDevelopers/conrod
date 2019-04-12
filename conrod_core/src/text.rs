//! Text layout logic.

use {FontSize, Scalar};
use std;

// Re-export all relevant rusttype types here.
pub use rusttype::{Glyph, GlyphId, GlyphIter, LayoutIter, Scale};
pub use rusttype::gpu_cache::Cache as GlyphCache;

/// Re-exported RustType geometrical types.
pub mod rt {
    pub use rusttype::{Point, Rect, Vector, gpu_cache, point, vector};
}


/// The RustType `FontCollection` type used by conrod.
pub type FontCollection = ::rusttype::FontCollection<'static>;
/// The RustType `Font` type used by conrod.
pub type Font = ::rusttype::Font<'static>;
/// The RustType `PositionedGlyph` type used by conrod.
pub type PositionedGlyph = ::rusttype::PositionedGlyph<'static>;

/// An iterator yielding each line within the given `text` as a new `&str`, where the start and end
/// indices into each line are provided by the given iterator.
#[derive(Clone)]
pub struct Lines<'a, I> {
    text: &'a str,
    ranges: I,
}

/// A type used for referring to typographic alignment of `Text`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Justify {
    /// Align text to the start of the bounding `Rect`'s *x* axis.
    Left,
    /// Symmetrically align text along the *y* axis.
    Center,
    /// Align text to the end of the bounding `Rect`'s *x* axis.
    Right,
    // /// Align wrapped text to both the start and end of the bounding `Rect`s *x* axis.
    // ///
    // /// Extra space is added between words in order to achieve this alignment.
    // TODO: Full,
}


/// Determine the total height of a block of text with the given number of lines, font size and
/// `line_spacing` (the space that separates each line of text).
pub fn height(num_lines: usize, font_size: FontSize, line_spacing: Scalar) -> Scalar {
    if num_lines > 0 {
        num_lines as Scalar * font_size as Scalar + (num_lines - 1) as Scalar * line_spacing
    } else {
        0.0
    }
}


/// Produce an iterator yielding each line within the given `text` as a new `&str`, where the
/// start and end indices into each line are provided by the given iterator.
pub fn lines<I>(text: &str, ranges: I) -> Lines<I>
    where I: Iterator<Item=std::ops::Range<usize>>,
{
    Lines {
        text: text,
        ranges: ranges,
    }
}


/// Converts the given font size in "points" to its font size in pixels.
/// This is useful for when the font size is not an integer.
pub fn f32_pt_to_px(font_size_in_points: f32) -> f32 {
    font_size_in_points * 4.0 / 3.0
}

/// Converts the given font size in "points" to a uniform `rusttype::Scale`.
/// This is useful for when the font size is not an integer.
pub fn f32_pt_to_scale(font_size_in_points: f32) -> Scale {
    Scale::uniform(f32_pt_to_px(font_size_in_points))
}

/// Converts the given font size in "points" to its font size in pixels.
pub fn pt_to_px(font_size_in_points: FontSize) -> f32 {
    f32_pt_to_px(font_size_in_points as f32)
}

/// Converts the given font size in "points" to a uniform `rusttype::Scale`.
pub fn pt_to_scale(font_size_in_points: FontSize) -> Scale {
    Scale::uniform(pt_to_px(font_size_in_points))
}


impl<'a, I> Iterator for Lines<'a, I>
    where I: Iterator<Item=std::ops::Range<usize>>,
{
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let Lines { text, ref mut ranges } = *self;
        ranges.next().map(|range| &text[range])
    }
}


/// The `font::Id` and `font::Map` types.
pub mod font {
    use fnv;
    use std;

    /// A type-safe wrapper around the `FontId`.
    ///
    /// This is used as both:
    ///
    /// - The key for the `font::Map`'s inner `HashMap`.
    /// - The `font_id` field for the rusttype::gpu_cache::Cache.
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Id(usize);

    /// A collection of mappings from `font::Id`s to `rusttype::Font`s.
    #[derive(Debug)]
    pub struct Map {
        next_index: usize,
        map: fnv::FnvHashMap<Id, super::Font>,
    }

    /// An iterator yielding an `Id` for each new `rusttype::Font` inserted into the `Map` via the
    /// `insert_collection` method.
    pub struct NewIds {
        index_range: std::ops::Range<usize>,
    }

    /// Yields the `Id` for each `Font` within the `Map`.
    #[derive(Clone)]
    pub struct Ids<'a> {
        keys: std::collections::hash_map::Keys<'a, Id, super::Font>,
    }

    /// Returned when loading new fonts from file or bytes.
    #[derive(Debug)]
    pub enum Error {
        /// Some error occurred while loading a `FontCollection` from a file.
        IO(std::io::Error),
        /// No `Font`s could be yielded from the `FontCollection`.
        NoFont,
    }

    impl Id {

        /// Returns the inner `usize` from the `Id`.
        pub fn index(self) -> usize {
            self.0
        }

    }

    impl Map {

        /// Construct the new, empty `Map`.
        pub fn new() -> Self {
            Map {
                next_index: 0,
                map: fnv::FnvHashMap::default(),
            }
        }

        /// Borrow the `rusttype::Font` associated with the given `font::Id`.
        pub fn get(&self, id: Id) -> Option<&super::Font> {
            self.map.get(&id)
        }

        /// Adds the given `rusttype::Font` to the `Map` and returns a unique `Id` for it.
        pub fn insert(&mut self, font: super::Font) -> Id {
            let index = self.next_index;
            self.next_index = index.wrapping_add(1);
            let id = Id(index);
            self.map.insert(id, font);
            id
        }

        /// Insert a single `Font` into the map by loading it from the given file path.
        pub fn insert_from_file<P>(&mut self, path: P) -> Result<Id, Error>
            where P: AsRef<std::path::Path>,
        {
            let font = from_file(path)?;
            Ok(self.insert(font))
        }

        // /// Adds each font in the given `rusttype::FontCollection` to the `Map` and returns an
        // /// iterator yielding a unique `Id` for each.
        // pub fn insert_collection(&mut self, collection: super::FontCollection) -> NewIds {
        //     let start_index = self.next_index;
        //     let mut end_index = start_index;
        //     for index in 0.. {
        //         match collection.font_at(index) {
        //             Some(font) => {
        //                 self.insert(font);
        //                 end_index += 1;
        //             }
        //             None => break,
        //         }
        //     }
        //     NewIds { index_range: start_index..end_index }
        // }

        /// Produces an iterator yielding the `Id` for each `Font` within the `Map`.
        pub fn ids(&self) -> Ids {
            Ids { keys: self.map.keys() }
        }

    }


    /// Load a `super::FontCollection` from a file at a given path.
    pub fn collection_from_file<P>(path: P) -> Result<super::FontCollection, std::io::Error>
        where P: AsRef<std::path::Path>,
    {
        use std::io::Read;
        let path = path.as_ref();
        let mut file = std::fs::File::open(path)?;
        let mut file_buffer = Vec::new();
        file.read_to_end(&mut file_buffer)?;
        Ok(super::FontCollection::from_bytes(file_buffer)?)
    }

    /// Load a single `Font` from a file at the given path.
    pub fn from_file<P>(path: P) -> Result<super::Font, Error>
        where P: AsRef<std::path::Path>
    {
        let collection = collection_from_file(path)?;
        collection.into_font().or(Err(Error::NoFont))
    }


    impl Iterator for NewIds {
        type Item = Id;
        fn next(&mut self) -> Option<Self::Item> {
            self.index_range.next().map(|i| Id(i))
        }
    }

    impl<'a> Iterator for Ids<'a> {
        type Item = Id;
        fn next(&mut self) -> Option<Self::Item> {
            self.keys.next().map(|&id| id)
        }
    }

    impl From<std::io::Error> for Error {
        fn from(e: std::io::Error) -> Self {
            Error::IO(e)
        }
    }

    impl std::error::Error for Error {
        fn description(&self) -> &str {
            match *self {
                Error::IO(ref e) => std::error::Error::description(e),
                Error::NoFont => "No `Font` found in the loaded `FontCollection`.",
            }
        }
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            match *self {
                Error::IO(ref e) => std::fmt::Display::fmt(e, f),
                _ => write!(f, "{}", std::error::Error::description(self))
            }
        }
    }

}


/// Logic and types specific to individual glyph layout.
pub mod glyph {
    use {FontSize, Range, Rect, Scalar};
    use std;

    /// Some position along the X axis (used within `CharXs`).
    pub type X = Scalar;

    /// The half of the width of some character (used within `CharXs`).
    pub type HalfW = Scalar;

    /// An iterator yielding the `Rect` for each `char`'s `Glyph` in the given `text`.
    pub struct Rects<'a, 'b> {
        /// The *y* axis `Range` of the `Line` for which character `Rect`s are being yielded.
        ///
        /// Every yielded `Rect` will use this as its `y` `Range`.
        y: Range,
        /// The position of the next `Rect`'s left edge along the *x* axis.
        next_left: Scalar,
        /// `PositionedGlyphs` yielded by the RustType `LayoutIter`.
        layout: super::LayoutIter<'a, 'b>,
    }

    /// An iterator that, for every `(line, line_rect)` pair yielded by the given iterator,
    /// produces an iterator that yields a `Rect` for every character in that line.
    pub struct RectsPerLine<'a, I> {
        lines_with_rects: I,
        font: &'a super::Font,
        font_size: FontSize,
    }

    /// Yields an iteraor yielding `Rect`s for each selected character in each line of text within
    /// the given iterator yielding char `Rect`s.
    ///
    /// Given some `start` and `end` indices, only `Rect`s for `char`s between these two indices
    /// will be produced.
    ///
    /// All lines that have no selected `Rect`s will be skipped.
    pub struct SelectedRectsPerLine<'a, I> {
        enumerated_rects_per_line: std::iter::Enumerate<RectsPerLine<'a, I>>,
        start_cursor_idx: super::cursor::Index,
        end_cursor_idx: super::cursor::Index,
    }

    /// Yields a `Rect` for each selected character in a single line of text.
    ///
    /// This iterator can only be produced by the `SelectedCharRectsPerLine` iterator.
    pub struct SelectedRects<'a, 'b> {
        enumerated_rects: std::iter::Enumerate<Rects<'a, 'b>>,
        end_char_idx: usize,
    }


    /// Find the index of the character that directly follows the cursor at the given `cursor_idx`.
    ///
    /// Returns `None` if either the given `cursor::Index` `line` or `idx` fields are out of bounds
    /// of the line information yielded by the `line_infos` iterator.
    pub fn index_after_cursor<I>(mut line_infos: I,
                                 cursor_idx: super::cursor::Index) -> Option<usize>
        where I: Iterator<Item=super::line::Info>,
    {
        line_infos
            .nth(cursor_idx.line)
            .and_then(|line_info| {
                let start_char = line_info.start_char;
                let end_char = line_info.end_char();
                let char_index = start_char + cursor_idx.char;
                if char_index <= end_char { Some(char_index) } else { None }
            })
    }

    /// Produce an iterator that, for every `(line, line_rect)` pair yielded by the given iterator,
    /// produces an iterator that yields a `Rect` for every character in that line.
    ///
    /// This is useful when information about character positioning is needed when reasoning about
    /// text layout.
    pub fn rects_per_line<'a, I>(lines_with_rects: I,
                                 font: &'a super::Font,
                                 font_size: FontSize) -> RectsPerLine<'a, I>
        where I: Iterator<Item=(&'a str, Rect)>,
    {
        RectsPerLine {
            lines_with_rects: lines_with_rects,
            font: font,
            font_size: font_size,
        }
    }

    /// Produces an iterator that yields iteraors yielding `Rect`s for each selected character in
    /// each line of text within the given iterator yielding char `Rect`s.
    ///
    /// Given some `start` and `end` indices, only `Rect`s for `char`s between these two indices
    /// will be produced.
    ///
    /// All lines that have no selected `Rect`s will be skipped.
    pub fn selected_rects_per_line<'a, I>(lines_with_rects: I,
                                          font: &'a super::Font,
                                          font_size: FontSize,
                                          start: super::cursor::Index,
                                          end: super::cursor::Index) -> SelectedRectsPerLine<'a, I>
        where I: Iterator<Item=(&'a str, Rect)>,
    {
        SelectedRectsPerLine {
            enumerated_rects_per_line:
                rects_per_line(lines_with_rects, font, font_size).enumerate(),
            start_cursor_idx: start,
            end_cursor_idx: end,
        }
    }

    impl<'a, I> Iterator for RectsPerLine<'a, I>
        where I: Iterator<Item=(&'a str, Rect)>,
    {
        type Item = Rects<'a, 'a>;
        fn next(&mut self) -> Option<Self::Item> {
            let RectsPerLine { ref mut lines_with_rects, font, font_size } = *self;
            let scale = super::pt_to_scale(font_size);
            lines_with_rects.next().map(|(line, line_rect)| {
                let (x, y) = (line_rect.left() as f32, line_rect.top() as f32);
                let point = super::rt::Point { x: x, y: y };
                Rects {
                    next_left: line_rect.x.start,
                    layout: font.layout(line, scale, point),
                    y: line_rect.y
                }
            })
        }
    }

    impl<'a, I> Iterator for SelectedRectsPerLine<'a, I>
        where I: Iterator<Item=(&'a str, Rect)>,
    {
        type Item = SelectedRects<'a, 'a>;
        fn next(&mut self) -> Option<Self::Item> {
            let SelectedRectsPerLine {
                ref mut enumerated_rects_per_line,
                start_cursor_idx,
                end_cursor_idx,
            } = *self;

            enumerated_rects_per_line.next().map(|(i, rects)| {
                let end_char_idx =
                    // If this is the last line, the end is the char after the final selected char.
                    if i == end_cursor_idx.line {
                        end_cursor_idx.char
                    // Otherwise if in range, every char in the line is selected.
                    } else if start_cursor_idx.line <= i && i < end_cursor_idx.line {
                        std::u32::MAX as usize
                    // Otherwise if out of range, no chars are selected.
                    } else {
                        0
                    };

                let mut enumerated_rects = rects.enumerate();

                // If this is the first line, skip all non-selected chars.
                if i == start_cursor_idx.line {
                    for _ in 0..start_cursor_idx.char {
                        enumerated_rects.next();
                    }
                }

                SelectedRects {
                    enumerated_rects: enumerated_rects,
                    end_char_idx: end_char_idx,
                }
            })
        }
    }

    impl<'a, 'b> Iterator for Rects<'a, 'b> {
        type Item = Rect;
        fn next(&mut self) -> Option<Self::Item> {
            let Rects { ref mut next_left, ref mut layout, y } = *self;
            layout.next().map(|g| {
                let left = *next_left;
                let right = g.pixel_bounding_box()
                    .map(|bb| bb.max.x as Scalar)
                    .unwrap_or_else(|| left + g.unpositioned().h_metrics().advance_width as Scalar);
                *next_left = right;
                let x = Range::new(left, right);
                Rect { x: x, y: y }
            })
        }
    }

    impl<'a, 'b> Iterator for SelectedRects<'a, 'b> {
        type Item = Rect;
        fn next(&mut self) -> Option<Self::Item> {
            let SelectedRects { ref mut enumerated_rects, end_char_idx } = *self;
            enumerated_rects.next()
                .and_then(|(i, rect)| {
                    if i < end_char_idx { Some(rect) }
                    else                { None }
                })
        }
    }

}


/// Logic related to the positioning of the cursor within text.
pub mod cursor {
    use FontSize;
    use position::{Range, Rect, Scalar, Point, Align};
    use std;

    /// Every possible cursor position within each line of text yielded by the given iterator.
    ///
    /// Yields `(xs, y_range)`, where `y_range` is the `Range` occupied by the line across the *y*
    /// axis and `xs` is every possible cursor position along the *x* axis
    #[derive(Clone)]
    pub struct XysPerLine<'a, I> {
        lines_with_rects: I,
        font: &'a super::Font,
        text: &'a str,
        font_size: FontSize,
    }

    /// Similarly to `XysPerLine`, yields every possible cursor position within each line of text
    /// yielded by the given iterator.
    ///
    /// Rather than taking an iterator type yielding lines and positioning data, this method
    /// constructs its own iterator to do so internally, saving some boilerplate involved in common
    /// `XysPerLine` use cases.
    ///
    /// Yields `(xs, y_range)`, where `y_range` is the `Range` occupied by the line across the *y*
    /// axis and `xs` is every possible cursor position along the *x* axis.
    #[derive(Clone)]
    pub struct XysPerLineFromText<'a> {
        xys_per_line: XysPerLine<'a,
            std::iter::Zip<std::iter::Cloned<std::slice::Iter<'a, super::line::Info>>,
            super::line::Rects<std::iter::Cloned<std::slice::Iter<'a, super::line::Info>>>>
        >,
    }

    /// Each possible cursor position along the *x* axis within a line of text.
    ///
    /// `Xs` iterators are produced by the `XysPerLine` iterator.
    pub struct Xs<'a, 'b> {
        next_x: Option<Scalar>,
        layout: super::LayoutIter<'a, 'b>,
    }

    /// An index representing the position of a cursor within some text.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Index {
        /// The index of the line upon which the cursor is situated.
        pub line: usize,
        /// The index within all possible cursor positions for the line.
        ///
        /// For example, for the line `foo`, a `char` of `1` would indicate the cursor's position
        /// as `f|oo` where `|` is the cursor.
        pub char: usize,
    }


    impl Index {

        /// The cursor index of the beginning of the word (block of non-whitespace) before `self`.
        ///
        /// If `self` is at the beginning of the line, call previous, which returns the last
        /// index position of the previous line, or None if it's the first line
        ///
        /// If `self` points to whitespace, skip past that whitespace, then return the index of 
        /// the start of the word that precedes the whitespace
        ///
        /// If `self` is in the middle or end of a word, return the index of the start of that word
        pub fn previous_word_start<I>(self, text: &str, mut line_infos: I) -> Option<Self>
            where I: Iterator<Item=super::line::Info>,
        {
            let Index { line, char } = self;
            if char > 0 {
                line_infos.nth(line).and_then(|line_info| {
                    let line_count = line_info.char_range().count();
                    let mut chars_rev = (&text[line_info.byte_range()]).chars().rev();
                    if char != line_count {
                        chars_rev.nth(line_count - char - 1);
                    }
                    let mut new_char = 0;
                    let mut hit_non_whitespace = false;
                    for (i, char_) in chars_rev.enumerate() {
                        // loop until word starts, then continue until the word ends
                        if !char_.is_whitespace() { hit_non_whitespace = true; }
                        if char_.is_whitespace() && hit_non_whitespace {
                            new_char = char - i;
                            break
                        }
                    }
                    Some(Index { line: line, char: new_char })
                })
            } else {
                self.previous(line_infos)
            }
        }

        /// The cursor index of the end of the first word (block of non-whitespace) after `self`.
        ///
        /// If `self` is at the end of the text, this returns `None`.
        ///
        /// If `self` is at the end of a line other than the last, this returns the first index of
        /// the next line.
        ///
        /// If `self` points to whitespace, skip past that whitespace, then return the index of 
        /// the end of the word after the whitespace
        ///
        /// If `self` is in the middle or start of a word, return the index of the end of that word
        pub fn next_word_end<I>(self, text: &str, mut line_infos: I) -> Option<Self>
            where I: Iterator<Item=super::line::Info>,
        {
            let Index { line, char } = self;
            line_infos.nth(line)
                .and_then(|line_info| {
                    let line_count = line_info.char_range().count();
                    if char < line_count {
                        let mut chars = (&text[line_info.byte_range()]).chars();
                        let mut new_char = line_count;
                        let mut hit_non_whitespace = false;
                        if char != 0 {
                            chars.nth(char - 1);
                        }
                        for (i, char_) in chars.enumerate() {
                            // loop until word starts, then continue until the word ends
                            if !char_.is_whitespace() { hit_non_whitespace = true; }
                            if char_.is_whitespace() && hit_non_whitespace {
                                new_char = char + i;
                                break
                            }
                        }
                        Some(Index { line: line, char: new_char })
                    } else {
                        line_infos.next().map(|_| Index { line: line + 1, char: 0 })
                    }
                })
        }

        /// The cursor index that comes before `self`.
        ///
        /// If `self` is at the beginning of the text, this returns `None`.
        ///
        /// If `self` is at the beginning of a line other than the first, this returns the last
        /// index position of the previous line.
        ///
        /// If `self` is a position other than the start of a line, it will return the position
        /// that is immediately to the left.
        pub fn previous<I>(self, mut line_infos: I) -> Option<Self>
            where I: Iterator<Item=super::line::Info>,
        {
            let Index { line, char } = self;
            if char > 0 {
                let new_char = char - 1;
                line_infos.nth(line)
                    .and_then(|info| if new_char <= info.char_range().count() {
                        Some(Index { line: line, char: new_char })
                    } else {
                        None
                    })
            } else if line > 0 {
                let new_line = line - 1;
                line_infos.nth(new_line)
                    .map(|info| {
                        let new_char = info.end_char() - info.start_char;
                        Index { line: new_line, char: new_char }
                    })
            } else {
                None
            }
        }

        /// The cursor index that follows `self`.
        ///
        /// If `self` is at the end of the text, this returns `None`.
        ///
        /// If `self` is at the end of a line other than the last, this returns the first index of
        /// the next line.
        ///
        /// If `self` is a position other than the end of a line, it will return the position that
        /// is immediately to the right.
        pub fn next<I>(self, mut line_infos: I) -> Option<Self>
            where I: Iterator<Item=super::line::Info>,
        {
            let Index { line, char } = self;
            line_infos.nth(line)
                .and_then(|info| {
                    if char >= info.char_range().count() {
                        line_infos.next().map(|_| Index { line: line + 1, char: 0 })
                    } else {
                        Some(Index { line: line, char: char + 1 })
                    }
                })
        }

        /// Clamps `self` to the given lines.
        ///
        /// If `self` would lie after the end of the last line, return the index at the end of the
        /// last line.
        ///
        /// If `line_infos` is empty, returns cursor at line=0 char=0.
        pub fn clamp_to_lines<I>(self, line_infos: I) -> Self
            where I: Iterator<Item=super::line::Info>,
        {
            let mut last = None;
            for (i, info) in line_infos.enumerate() {
                if i == self.line {
                    let num_chars = info.char_range().len();
                    let char = std::cmp::min(self.char, num_chars);
                    return Index { line: i, char: char };
                }
                last = Some((i, info));
            }
            match last {
                Some((i, info)) => Index {
                    line: i,
                    char: info.char_range().len(),
                },
                None => Index { line: 0, char: 0 },
            }
        }

    }


    /// Every possible cursor position within each line of text yielded by the given iterator.
    ///
    /// Yields `(xs, y_range)`, where `y_range` is the `Range` occupied by the line across the *y*
    /// axis and `xs` is every possible cursor position along the *x* axis
    pub fn xys_per_line<'a, I>(lines_with_rects: I,
                               font: &'a super::Font,
                               text: &'a str,
                               font_size: FontSize) -> XysPerLine<'a, I>
    {
        XysPerLine {
            lines_with_rects: lines_with_rects,
            font: font,
            text: text,
            font_size: font_size,
        }
    }

    /// Similarly to `xys_per_line`, this produces an iterator yielding every possible cursor
    /// position within each line of text yielded by the given iterator.
    ///
    /// Rather than taking an iterator yielding lines and their positioning data, this method
    /// constructs its own iterator to do so internally, saving some boilerplate involved in common
    /// `xys_per_line` use cases.
    ///
    /// Yields `(xs, y_range)`, where `y_range` is the `Range` occupied by the line across the *y*
    /// axis and `xs` is every possible cursor position along the *x* axis.
    pub fn xys_per_line_from_text<'a>(text: &'a str,
                                      line_infos: &'a [super::line::Info],
                                      font: &'a super::Font,
                                      font_size: FontSize,
                                      x_align: super::Justify,
                                      y_align: Align,
                                      line_spacing: Scalar,
                                      rect: Rect) -> XysPerLineFromText<'a>
    {
        let line_infos = line_infos.iter().cloned();
        let line_rects = super::line::rects(line_infos.clone(), font_size, rect,
                                            x_align, y_align, line_spacing);
        let lines = line_infos.clone();
        let lines_with_rects = lines.zip(line_rects.clone());
        XysPerLineFromText {
            xys_per_line: super::cursor::xys_per_line(lines_with_rects, font, text, font_size),
        }
    }

    /// Convert the given character index into a cursor `Index`.
    pub fn index_before_char<I>(line_infos: I, char_index: usize) -> Option<Index>
        where I: Iterator<Item=super::line::Info>,
    {
        for (i, line_info) in line_infos.enumerate() {
            let start_char = line_info.start_char;
            let end_char = line_info.end_char();
            if start_char <= char_index && char_index <= end_char {
                return Some(Index { line: i, char: char_index - start_char });
            }
        }
        None
    }

    /// Determine the *xy* location of the cursor at the given cursor `Index`.
    pub fn xy_at<'a, I>(xys_per_line: I, idx: Index) -> Option<(Scalar, Range)>
        where I: Iterator<Item=(Xs<'a, 'a>, Range)>,
    {
        for (i, (xs, y)) in xys_per_line.enumerate() {
            if i == idx.line {
                for (j, x) in xs.enumerate() {
                    if j == idx.char {
                        return Some((x, y));
                    }
                }
            }
        }
        None
    }

    /// Find the closest line for the given `y` position, and return the line index, Xs iterator, and y-range of that line
    ///
    /// Returns `None` if there are no lines
    pub fn closest_line<'a, I>(y_pos: Scalar, xys_per_line: I) -> Option<(usize, Xs<'a,'a>, Range)>
        where I: Iterator<Item = (Xs<'a, 'a>, Range)>,
    {
        let mut xys_per_line_enumerated = xys_per_line.enumerate();
        xys_per_line_enumerated.next().and_then(|(first_line_idx, (first_line_xs, first_line_y))| {
            let mut closest_line = (first_line_idx,first_line_xs,first_line_y);
            let mut closest_diff = (y_pos - first_line_y.middle()).abs();
            for (line_idx, (line_xs, line_y)) in xys_per_line_enumerated {
                if line_y.is_over(y_pos) {
                    closest_line = (line_idx,line_xs,line_y);
                    break;
                } else {
                    let diff = (y_pos - line_y.middle()).abs();
                    if diff < closest_diff {
                        closest_line = (line_idx,line_xs,line_y);
                        closest_diff = diff;
                    } else {
                        break;
                    }
                }
            }
            Some(closest_line)
        })
    }

    /// Find the closest cursor index to the given `xy` position, and the center `Point` of that
    /// cursor.
    ///
    /// Returns `None` if the given `text` is empty.
    pub fn closest_cursor_index_and_xy<'a, I>(xy: Point, xys_per_line: I) -> Option<(Index, Point)>
        where I: Iterator<Item = (Xs<'a, 'a>, Range)>,
    {
        closest_line(xy[1], xys_per_line)
            .and_then(|(closest_line_idx, closest_line_xs, closest_line_y)| {
                let (closest_char_idx, closest_x) =
                    closest_cursor_index_on_line(xy[0], closest_line_xs);
                let index = Index {
                    line: closest_line_idx,
                    char: closest_char_idx,
                };
                let point = [closest_x, closest_line_y.middle()];
                Some((index, point))
            })
    }

    /// Find the closest cursor index to the given `x` position on the given line along with the
    /// `x` position of that cursor.
    pub fn closest_cursor_index_on_line<'a>(x_pos: Scalar, line_xs: Xs<'a, 'a>) -> (usize, Scalar) {
        let mut xs_enumerated = line_xs.enumerate();
        // `xs` always yields at least one `x` (the start of the line).
        let (first_idx, first_x) = xs_enumerated.next().unwrap();
        let first_diff = (x_pos - first_x).abs();
        let mut closest = (first_idx,first_x);
        let mut closest_diff = first_diff;
        for (i, x) in xs_enumerated {
            let diff = (x_pos - x).abs();
            if diff < closest_diff {
                closest = (i,x);
                closest_diff = diff;
            } else {
                break;
            }
        }
        closest
    }


    impl<'a, I> Iterator for XysPerLine<'a, I>
        where I: Iterator<Item=(super::line::Info, Rect)>,
    {
        // The `Range` occupied by the line across the *y* axis, along with an iterator yielding
        // each possible cursor position along the *x* axis.
        type Item = (Xs<'a, 'a>, Range);
        fn next(&mut self) -> Option<Self::Item> {
            let XysPerLine { ref mut lines_with_rects, font, text, font_size } = *self;
            let scale = super::pt_to_scale(font_size);
            lines_with_rects.next().map(|(line_info, line_rect)| {
                let line = &text[line_info.byte_range()];
                let (x, y) = (line_rect.left() as f32, line_rect.top() as f32);
                let point = super::rt::Point { x: x, y: y };
                let y = line_rect.y;
                let layout = font.layout(line, scale, point);
                let xs = Xs {
                    next_x: Some(line_rect.x.start),
                    layout: layout,
                };
                (xs, y)
            })
        }
    }

    impl<'a> Iterator for XysPerLineFromText<'a> {
        type Item = (Xs<'a, 'a>, Range);
        fn next(&mut self) -> Option<Self::Item> {
            self.xys_per_line.next()
        }
    }

    impl<'a, 'b> Iterator for Xs<'a, 'b> {
        // Each possible cursor position along the *x* axis.
        type Item = Scalar;
        fn next(&mut self) -> Option<Self::Item> {
            self.next_x.map(|x| {
                self.next_x = self.layout.next()
                    .map(|g| {
                        g.pixel_bounding_box()
                            .map(|r| r.max.x as Scalar)
                            .unwrap_or_else(|| {
                                x + g.unpositioned().h_metrics().advance_width as Scalar
                            })
                    });
                x
            })
        }
    }
}


/// Text handling logic related to individual lines of text.
///
/// This module is the core of multi-line text handling.
pub mod line {
    use FontSize;
    use position::{Align, Range, Rect, Scalar};
    use std;

    /// The two types of **Break** indices returned by the **WrapIndicesBy** iterators.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum Break {
        /// A break caused by the text exceeding some maximum width.
        Wrap {
            /// The byte index at which the break occurs.
            byte: usize,
            /// The char index at which the string should wrap due to exceeding a maximum width.
            char: usize,
            /// The byte length which should be skipped in order to reach the first non-whitespace
            /// character to use as the beginning of the next line.
            len_bytes: usize,
        },
        /// A break caused by a newline character.
        Newline {
            /// The byte index at which the string should wrap due to exceeding a maximum width.
            byte: usize,
            /// The char index at which the string should wrap due to exceeding a maximum width.
            char: usize,
            /// The width of the "newline" token in bytes.
            len_bytes: usize,
        },
        /// The end of the string has been reached, with the given length.
        End {
            /// The ending byte index.
            byte: usize,
            /// The ending char index.
            char: usize,
        },
    }

    /// Information about a single line of text within a `&str`.
    ///
    /// `Info` is a minimal amount of information that can be stored for efficient reasoning about
    /// blocks of text given some `&str`. The `start` and `end_break` can be used for indexing into
    /// the `&str`, and the `width` can be used for calculating line `Rect`s, alignment, etc.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Info {
        /// The index into the `&str` that represents the first character within the line.
        pub start_byte: usize,
        /// The character index of the first character in the line.
        pub start_char: usize,
        /// The index within the `&str` at which this line breaks into a new line, along with the
        /// index at which the following line begins. The variant describes whether the break is
        /// caused by a `Newline` character or a `Wrap` by the given wrap function.
        pub end_break: Break,
        /// The total width of all characters within the line.
        pub width: Scalar,
    }

    /// An iterator yielding an `Info` struct for each line in the given `text` wrapped by the
    /// given `next_break_fn`.
    ///
    /// `Infos` is a fundamental part of performing lazy reasoning about text within conrod.
    ///
    /// Construct an `Infos` iterator via the [infos function](./fn.infos.html) and its two builder
    /// methods, [wrap_by_character](./struct.Infos.html#method.wrap_by_character) and
    /// [wrap_by_whitespace](./struct.Infos.html#method.wrap_by_whitespace).
    pub struct Infos<'a, F> {
        text: &'a str,
        font: &'a super::Font,
        font_size: FontSize,
        max_width: Scalar,
        next_break_fn: F,
        /// The index that indicates the start of the next line to be yielded.
        start_byte: usize,
        /// The character index that indicates the start of the next line to be yielded.
        start_char: usize,
        /// The break type of the previously yielded line
        last_break: Option<Break>,
    }

    /// An iterator yielding a `Rect` for each line in 
    #[derive(Clone)]
    pub struct Rects<I> {
        infos: I,
        x_align: super::Justify,
        line_spacing: Scalar,
        next: Option<Rect>,
    }

    /// An iterator yielding a `Rect` for each selected line in a block of text.
    ///
    /// The yielded `Rect`s represent the selected range within each line of text.
    ///
    /// Lines that do not contain any selected text will be skipped.
    pub struct SelectedRects<'a, I> {
        selected_char_rects_per_line: super::glyph::SelectedRectsPerLine<'a, I>,
    }

    /// An alias for function pointers that are compatible with the `Block`'s required text
    /// wrapping function.
    pub type NextBreakFnPtr = fn(&str, &super::Font, FontSize, Scalar) -> (Break, Scalar);


    impl Break {

        /// Return the index at which the break occurs.
        pub fn byte_index(self) -> usize {
            match self {
                Break::Wrap { byte, .. } |
                Break::Newline { byte, .. } |
                Break::End { byte, .. } => byte,
            }
        }

        /// Return the index of the `char` at which the break occurs.
        ///
        /// To clarify, this index is to be used in relation to the `Chars` iterator.
        pub fn char_index(self) -> usize {
            match self {
                Break::Wrap { char, .. } |
                Break::Newline { char, .. } |
                Break::End { char, .. } => char,
            }
        }

    }

    impl<'a, F> Clone for Infos<'a, F>
        where F: Clone,
    {
        fn clone(&self) -> Self {
            Infos {
                text: self.text,
                font: self.font,
                font_size: self.font_size,
                max_width: self.max_width,
                next_break_fn: self.next_break_fn.clone(),
                start_byte: self.start_byte,
                start_char: self.start_char,
                last_break: None,
            }
        }
    }

    impl Info {

        /// The end of the byte index range for indexing into the slice.
        pub fn end_byte(&self) -> usize {
            self.end_break.byte_index()
        }

        /// The end of the index range for indexing into the slice.
        pub fn end_char(&self) -> usize {
            self.end_break.char_index()
        }

        /// The index range for indexing (via bytes) into the original str slice.
        pub fn byte_range(self) -> std::ops::Range<usize> {
            self.start_byte..self.end_byte()
        }

        /// The index range for indexing into a `char` iterator over the original str slice.
        pub fn char_range(self) -> std::ops::Range<usize> {
            self.start_char..self.end_char()
        }

    }

    impl<'a> Infos<'a, NextBreakFnPtr> {

        /// Converts `Self` into an `Infos` whose lines are wrapped at the character that first
        /// causes the line width to exceed the given `max_width`.
        pub fn wrap_by_character(mut self, max_width: Scalar) -> Self {
            self.next_break_fn = next_break_by_character;
            self.max_width = max_width;
            self
        }

        /// Converts `Self` into an `Infos` whose lines are wrapped at the whitespace prior to the
        /// character that causes the line width to exceed the given `max_width`.
        pub fn wrap_by_whitespace(mut self, max_width: Scalar) -> Self {
            self.next_break_fn = next_break_by_whitespace;
            self.max_width = max_width;
            self
        }

    }


    /// A function for finding the advance width between the given character that also considers
    /// the kerning for some previous glyph.
    ///
    /// This also updates the `last_glyph` with the glyph produced for the given `char`.
    ///
    /// This is primarily for use within the `next_break` functions below.
    ///
    /// The following code is adapted from the rusttype::LayoutIter::next src.
    fn advance_width(ch: char,
                     font: &super::Font,
                     scale: super::Scale,
                     last_glyph: &mut Option<super::GlyphId>) -> Scalar
    {
        let g = font.glyph(ch).scaled(scale);
        let kern = last_glyph
            .map(|last| font.pair_kerning(scale, last, g.id()))
            .unwrap_or(0.0);
        let advance_width = g.h_metrics().advance_width;
        *last_glyph = Some(g.id());
        (kern + advance_width) as Scalar
    }


    /// Returns the next index at which the text naturally breaks via a newline character,
    /// along with the width of the line.
    fn next_break(text: &str,
                  font: &super::Font,
                  font_size: FontSize) -> (Break, Scalar)
    {
        let scale = super::pt_to_scale(font_size);
        let mut width = 0.0;
        let mut char_i = 0;
        let mut char_indices = text.char_indices().peekable();
        let mut last_glyph = None;
        while let Some((byte_i, ch)) = char_indices.next() {
            // Check for a newline.
            if ch == '\r' {
                if let Some(&(_, '\n')) = char_indices.peek() {
                    let break_ = Break::Newline { byte: byte_i, char: char_i, len_bytes: 2 };
                    return (break_, width);
                }
            } else if ch == '\n' {
                let break_ = Break::Newline { byte: byte_i, char: char_i, len_bytes: 1 };
                return (break_, width);
            }

            // Update the width.
            width += advance_width(ch, font, scale, &mut last_glyph);
            char_i += 1;
        }
        let break_ = Break::End { byte: text.len(), char: char_i };
        (break_, width)
    }

    /// Returns the next index at which the text will break by either:
    /// - A newline character.
    /// - A line wrap at the beginning of the first character exceeding the `max_width`.
    ///
    /// Also returns the width of each line alongside the Break.
    fn next_break_by_character(text: &str,
                               font: &super::Font,
                               font_size: FontSize,
                               max_width: Scalar) -> (Break, Scalar)
    {
        let scale = super::pt_to_scale(font_size);
        let mut width = 0.0;
        let mut char_i = 0;
        let mut char_indices = text.char_indices().peekable();
        let mut last_glyph = None;
        while let Some((byte_i, ch)) = char_indices.next() {

            // Check for a newline.
            if ch == '\r' {
                if let Some(&(_, '\n')) = char_indices.peek() {
                    let break_ = Break::Newline { byte: byte_i, char: char_i, len_bytes: 2 };
                    return (break_, width);
                }
            } else if ch == '\n' {
                let break_ = Break::Newline { byte: byte_i, char: char_i, len_bytes: 1 };
                return (break_, width);
            }

            // Add the character's width to the width so far.
            let new_width = width + advance_width(ch, font, scale, &mut last_glyph);

            // Check for a line wrap.
            if new_width > max_width {
                let break_ = Break::Wrap { byte: byte_i, char: char_i, len_bytes: 0 };
                return (break_, width);
            }

            width = new_width;
            char_i += 1;
        }

        let break_ = Break::End { byte: text.len(), char: char_i };
        (break_, width)
    }

    /// Returns the next index at which the text will break by either:
    /// - A newline character.
    /// - A line wrap at the beginning of the whitespace that preceeds the first word
    /// exceeding the `max_width`.
    /// - A line wrap at the beginning of the first character exceeding the `max_width`,
    /// if no whitespace appears for `max_width` characters.
    ///
    /// Also returns the width the line alongside the Break.
    fn next_break_by_whitespace(text: &str,
                                font: &super::Font,
                                font_size: FontSize,
                                max_width: Scalar) -> (Break, Scalar)
    {
        struct Last { byte: usize, char: usize, width_before: Scalar }
        let scale = super::pt_to_scale(font_size);
        let mut last_whitespace_start = None;
        let mut width = 0.0;
        let mut char_i = 0;
        let mut char_indices = text.char_indices().peekable();
        let mut last_glyph = None;
        while let Some((byte_i, ch)) = char_indices.next() {

            // Check for a newline.
            if ch == '\r' {
                if let Some(&(_, '\n')) = char_indices.peek() {
                    let break_ = Break::Newline { byte: byte_i, char: char_i, len_bytes: 2 };
                    return (break_, width)
                }
            } else if ch == '\n' {
                let break_ = Break::Newline { byte: byte_i, char: char_i, len_bytes: 1 };
                return (break_, width);
            }

            // Add the character's width to the width so far.
            let new_width = width + advance_width(ch, font, scale, &mut last_glyph);

            // Check for a line wrap.
            if width > max_width {
                match last_whitespace_start {
                    Some(Last { byte, char, width_before }) => {
                        let break_ = Break::Wrap { byte: byte, char: char, len_bytes: 1 };
                        return (break_, width_before);
                    },
                    None => {
                        let break_ = Break::Wrap { byte: byte_i, char: char_i, len_bytes: 0 };
                        return (break_, width);
                    }
                }
            }

            // Check for a new whitespace.
            if ch.is_whitespace() {
                last_whitespace_start = Some(Last { byte: byte_i, char: char_i, width_before: width });
            }

            width = new_width;
            char_i += 1;
        }

        let break_ = Break::End { byte: text.len(), char: char_i };
        (break_, width)
    }


    /// Produce the width of the given line of text including spaces (i.e. ' ').
    pub fn width(text: &str, font: &super::Font, font_size: FontSize) -> Scalar {
        let scale = super::Scale::uniform(super::pt_to_px(font_size));
        let point = super::rt::Point { x: 0.0, y: 0.0 };

        let mut total_w = 0.0;
        for g in font.layout(text, scale, point) {
            match g.pixel_bounding_box() {
                Some(bb) => total_w = bb.max.x as f32,
                None => total_w += g.unpositioned().h_metrics().advance_width,
            }
        }

        total_w as Scalar
    }


    /// Produce an `Infos` iterator wrapped by the given `next_break_fn`.
    pub fn infos_wrapped_by<'a, F>(text: &'a str,
                                   font: &'a super::Font,
                                   font_size: FontSize,
                                   max_width: Scalar,
                                   next_break_fn: F) -> Infos<'a, F>
        where F: for<'b> FnMut(&'b str, &'b super::Font, FontSize, Scalar) -> (Break, Scalar)
    {
        Infos {
            text: text,
            font: font,
            font_size: font_size,
            max_width: max_width,
            next_break_fn: next_break_fn,
            start_byte: 0,
            start_char: 0,
            last_break: None,
        }
    }

    /// Produce an `Infos` iterator that yields an `Info` for every line in the given text.
    ///
    /// The produced `Infos` iterator will not wrap the text, and only break each line via newline
    /// characters within the text (either `\n` or `\r\n`).
    pub fn infos<'a>(text: &'a str,
                     font: &'a super::Font,
                     font_size: FontSize) -> Infos<'a, NextBreakFnPtr>
    {
        fn no_wrap(text: &str,
                   font: &super::Font,
                   font_size: FontSize,
                   _max_width: Scalar) -> (Break, Scalar)
        {
            next_break(text, font, font_size)
        }

        infos_wrapped_by(text, font, font_size, std::f64::MAX, no_wrap)
    }

    /// Produce an iterator yielding the bounding `Rect` for each line in the text.
    ///
    /// This function assumes that `font_size` is the same `FontSize` used to produce the `Info`s
    /// yielded by the `infos` Iterator.
    pub fn rects<I>(mut infos: I,
                    font_size: FontSize,
                    bounding_rect: Rect,
                    x_align: super::Justify,
                    y_align: Align,
                    line_spacing: Scalar) -> Rects<I>
        where I: Iterator<Item=Info> + ExactSizeIterator,
    {
        let num_lines = infos.len();
        let first_rect = infos.next().map(|first_info| {

            // Calculate the `x` `Range` of the first line `Rect`.
            let range = Range::new(0.0, first_info.width);
            let x = match x_align {
                super::Justify::Left => range.align_start_of(bounding_rect.x),
                super::Justify::Center => range.align_middle_of(bounding_rect.x),
                super::Justify::Right => range.align_end_of(bounding_rect.x),
            };

            // Calculate the `y` `Range` of the first line `Rect`.
            let total_text_height = super::height(num_lines, font_size, line_spacing);
            let total_text_y_range = Range::new(0.0, total_text_height);
            let total_text_y = match y_align {
                Align::Start => total_text_y_range.align_start_of(bounding_rect.y),
                Align::Middle => total_text_y_range.align_middle_of(bounding_rect.y),
                Align::End => total_text_y_range.align_end_of(bounding_rect.y),
            };
            let range = Range::new(0.0, font_size as Scalar);
            let y = range.align_end_of(total_text_y);

            Rect { x: x, y: y }
        });

        Rects {
            infos: infos,
            next: first_rect,
            x_align: x_align,
            line_spacing: line_spacing,
        }
    }

    /// Produces an iterator yielding a `Rect` for the selected range in each selected line in a block
    /// of text.
    ///
    /// The yielded `Rect`s represent the selected range within each line of text.
    ///
    /// Lines that do not contain any selected text will be skipped.
    pub fn selected_rects<'a, I>(lines_with_rects: I,
                                 font: &'a super::Font,
                                 font_size: FontSize,
                                 start: super::cursor::Index,
                                 end: super::cursor::Index) -> SelectedRects<'a, I>
        where I: Iterator<Item=(&'a str, Rect)>,
    {
        SelectedRects {
            selected_char_rects_per_line:
                super::glyph::selected_rects_per_line(lines_with_rects, font, font_size, start, end)
        }
    }


    impl<'a, F> Iterator for Infos<'a, F>
        where F: for<'b> FnMut(&'b str, &'b super::Font, FontSize, Scalar) -> (Break, Scalar)
    {
        type Item = Info;
        fn next(&mut self) -> Option<Self::Item> {
            let Infos {
                text,
                font,
                font_size,
                max_width,
                ref mut next_break_fn,
                ref mut start_byte,
                ref mut start_char,
                ref mut last_break,
            } = *self;

            match next_break_fn(&text[*start_byte..], font, font_size, max_width) {
                (next @ Break::Newline { .. }, width) | (next @ Break::Wrap { .. }, width) => {

                    let next_break = match next {
                        Break::Newline { byte, char, len_bytes } =>
                            Break::Newline {
                                byte: *start_byte + byte,
                                char: *start_char + char,
                                len_bytes: len_bytes,
                            },
                        Break::Wrap { byte, char, len_bytes } =>
                            Break::Wrap {
                                byte: *start_byte + byte,
                                char: *start_char + char,
                                len_bytes: len_bytes,
                            },
                        _ => unreachable!(),
                    };

                    let info = Info {
                        start_byte: *start_byte,
                        start_char: *start_char,
                        end_break: next_break,
                        width: width,
                    };

                    match next {
                        Break::Newline { byte, char, len_bytes } |
                        Break::Wrap { byte, char, len_bytes } => {
                            *start_byte = info.start_byte + byte + len_bytes;
                            *start_char = info.start_char + char + 1;
                        },
                        _ => unreachable!(),
                    };
                    *last_break = Some(next_break);
                    Some(info)
                },

                (Break::End { char, .. }, width) => {
                    // if the last line ends in a new line, or the entire text is empty, return an empty line Info
                    let empty_line = {
                        match *last_break {
                            Some(last_break_) => match last_break_ {
                                Break::Newline { .. } => true,
                                _ => false,
                            }, None => true,
                        }
                    };
                    if *start_byte < text.len() || empty_line {
                        let total_bytes = text.len();
                        let total_chars = *start_char + char;
                        let end_break = Break::End {
                            byte: total_bytes,
                            char: total_chars,
                        };
                        let info = Info {
                            start_byte: *start_byte,
                            start_char: *start_char,
                            end_break: end_break,
                            width: width,
                        };
                        *start_byte = total_bytes;
                        *start_char = total_chars;
                        *last_break = Some(end_break);
                        Some(info)
                    } else {
                        None
                    }
                },
            }
        }
    }

    impl<I> Iterator for Rects<I>
        where I: Iterator<Item=Info>,
    {
        type Item = Rect;
        fn next(&mut self) -> Option<Self::Item> {
            let Rects { ref mut next, ref mut infos, x_align, line_spacing } = *self;
            next.map(|line_rect| {
                *next = infos.next().map(|info| {

                    let y = {
                        let h = line_rect.h();
                        let y = line_rect.y() - h - line_spacing;
                        Range::from_pos_and_len(y, h)
                    };

                    let x = {
                        let range = Range::new(0.0, info.width);
                        match x_align {
                            super::Justify::Left => range.align_start_of(line_rect.x),
                            super::Justify::Center => range.align_middle_of(line_rect.x),
                            super::Justify::Right => range.align_end_of(line_rect.x),
                        }
                    };

                    Rect { x: x, y: y }
                });

                line_rect
            })
        }
    }

    impl<'a, I> Iterator for SelectedRects<'a, I>
        where I: Iterator<Item=(&'a str, Rect)>,
    {
        type Item = Rect;
        fn next(&mut self) -> Option<Self::Item> {
            while let Some(mut rects) = self.selected_char_rects_per_line.next() {
                if let Some(first_rect) = rects.next() {
                    let total_selected_rect = rects.fold(first_rect, |mut total, next| {
                        total.x.end = next.x.end;
                        total
                    });
                    return Some(total_selected_rect);
                }
            }
            None
        }
    }

}
