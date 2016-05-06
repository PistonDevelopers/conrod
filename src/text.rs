//! Text layout logic.

use {FontSize, Scalar};
use std;

/// An iterator yielding each line within the given `text` as a new `&str`, where the start and end
/// indices into each line are provided by the given iterator.
#[derive(Clone)]
pub struct Lines<'a, I> {
    text: &'a str,
    ranges: I,
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


impl<'a, I> Iterator for Lines<'a, I>
    where I: Iterator<Item=std::ops::Range<usize>>,
{
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let Lines { text, ref mut ranges } = *self;
        ranges.next().map(|range| &text[range])
    }
}


/// Logic and types specific to individual character layout.
pub mod char {
    use {CharacterCache, FontSize, GlyphCache, Range, Rect, Scalar};
    use std;

    /// Some position along the X axis (used within `CharXs`).
    pub type X = Scalar;

    /// The half of the width of some character (used within `CharXs`).
    pub type HalfW = Scalar;

    /// An iterator yielding the widths of each consecutive character in some sequence.
    #[derive(Clone)]
    pub struct Widths<'a, I, C: 'a> {
        chars: I,
        cache: &'a GlyphCache<C>,
        font_size: FontSize,
    }

    /// An iterator yielding the `Rect` for each `char` in the given `text`.
    #[derive(Clone)]
    pub struct Rects<'a, C: 'a> {
        /// The *y* axis `Range` of the `Line` for which character `Rect`s are being yielded.
        ///
        /// Every yielded `Rect` will use this as its `y` `Range`.
        y: Range,
        /// The position of the next `Rect`'s left edge along the *x* axis.
        next_left: Scalar,
        widths: Widths<'a, std::str::Chars<'a>, C>,
    }

    /// An iterator that, for every `(line, line_rect)` pair yielded by the given iterator,
    /// produces an iterator that yields a `Rect` for every character in that line.
    #[derive(Clone)]
    pub struct RectsPerLine<'a, I, C: 'a> {
        lines_with_rects: I,
        cache: &'a GlyphCache<C>,
        font_size: FontSize,
    }


    /// Converts the given sequence of `char`s into their Scalar widths.
    pub fn widths<I, C>(chars: I,
                        cache: &GlyphCache<C>,
                        font_size: FontSize) -> Widths<I::IntoIter, C>
        where I: IntoIterator<Item=char>,
              C: CharacterCache,
    {
        Widths {
            chars: chars.into_iter(),
            cache: cache,
            font_size: font_size,
        }
    }

    /// Produce an iterator that, for every `(line, line_rect)` pair yielded by the given iterator,
    /// produces an iterator that yields a `Rect` for every character in that line.
    ///
    /// This is useful when information about character positioning is needed when reasoning about
    /// text layout.
    pub fn rects_per_line<'a, I, C>(lines_with_rects: I,
                                    cache: &'a GlyphCache<C>,
                                    font_size: FontSize) -> RectsPerLine<'a, I, C>
    {
        RectsPerLine {
            lines_with_rects: lines_with_rects,
            cache: cache,
            font_size: font_size,
        }
    }

    /// Find the index of the character that directly follows the cursor at the given `cursor_idx`.
    ///
    /// Returns `None` if either the given `cursor::Index` `line` or `idx` fields are out of bounds
    /// of the `line_infos` iterator.
    pub fn index_after_cursor<I>(mut line_infos: I, cursor: super::cursor::Index) -> Option<usize>
        where I: Iterator<Item=super::line::Info>,
    {
        line_infos.nth(cursor.line).and_then(|line_info| {
            let start = line_info.start;
            let end = line_info.end();
            let char_index = start + cursor.idx;
            if start <= char_index && char_index <= end { Some(char_index) } else { None }
        })
    }


    impl<'a, I, C> Iterator for Widths<'a, I, C>
        where I: Iterator<Item=char>,
              C: CharacterCache,
    {
        type Item = Scalar;
        fn next(&mut self) -> Option<Self::Item> {
            let Widths { font_size, cache, ref mut chars } = *self;
            chars.next().map(|ch| cache.char_width(font_size, ch))
        }
    }

    impl<'a, I, C> Iterator for RectsPerLine<'a, I, C>
        where I: Iterator<Item=(&'a str, Rect)>,
              C: CharacterCache,
    {
        type Item = Rects<'a, C>;
        fn next(&mut self) -> Option<Self::Item> {
            let RectsPerLine { ref mut lines_with_rects, cache, font_size } = *self;
            lines_with_rects.next().map(|(line, line_rect)| {
                Rects {
                    next_left: line_rect.x.start,
                    widths: widths(line.chars(), cache, font_size),
                    y: line_rect.y
                }
            })
        }
    }

    impl<'a, C> Iterator for Rects<'a, C>
        where C: CharacterCache,
    {
        type Item = Rect;
        fn next(&mut self) -> Option<Self::Item> {
            let Rects { ref mut next_left, ref mut widths, y } = *self;
            widths.next().map(|w| {
                let left = *next_left;
                let right = left + w;
                *next_left = right;
                let x = Range::new(left, right);
                Rect { x: x, y: y }
            })
        }
    }

}


/// Logic related to the positioning of the cursor within text.
pub mod cursor {
    use {CharacterCache, FontSize, GlyphCache, Range, Rect, Scalar};
    use std;

    /// Every possible cursor position within each line of text yielded by the given iterator.
    ///
    /// Yields `(xs, y_range)`, where `y_range` is the `Range` occupied by the line across the *y*
    /// axis and `xs` is every possible cursor position along the *x* axis
    #[derive(Clone)]
    pub struct XysPerLine<'a, I, C: 'a> {
        lines_with_rects: I,
        cache: &'a GlyphCache<C>,
        font_size: FontSize,
    }

    /// Each possible cursor position along the *x* axis within a line of text.
    ///
    /// `Xs` iterators are produced by the `XysPerLine` iterator.
    #[derive(Clone)]
    pub struct Xs<'a, C: 'a> {
        next_x: Option<Scalar>,
        widths: super::char::Widths<'a, std::str::Chars<'a>, C>,
    }

    /// An index representing the position of a cursor within some text.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Index {
        /// The index of the line upon which the cursor is situated.
        pub line: usize,
        /// The index within all possible cursor positions for the line.
        pub idx: usize,
    }


    /// Every possible cursor position within each line of text yielded by the given iterator.
    ///
    /// Yields `(xs, y_range)`, where `y_range` is the `Range` occupied by the line across the *y*
    /// axis and `xs` is every possible cursor position along the *x* axis
    pub fn xys_per_line<I, C>(lines_with_rects: I,
                              cache: &GlyphCache<C>,
                              font_size: FontSize) -> XysPerLine<I, C>
    {
        XysPerLine {
            lines_with_rects: lines_with_rects,
            cache: cache,
            font_size: font_size,
        }
    }

    /// Convert the given character index into a cursor `Index`.
    pub fn index_before_char<I>(line_infos: I, char_index: usize) -> Option<Index>
        where I: Iterator<Item=super::line::Info>,
    {
        for (i, line_info) in line_infos.enumerate() {
            let start = line_info.start;
            let end = line_info.end();
            if start <= char_index && char_index <= end+1 {
                return Some(Index { line: i, idx: char_index - start });
            }
        }
        None
    }

    /// Determine the *xy* location of the cursor at the given cursor `Index`.
    pub fn xy_at<'a, I, C: 'a>(xys_per_line: I, idx: Index) -> Option<(Scalar, Range)>
        where I: Iterator<Item=(Xs<'a, C>, Range)>,
              C: CharacterCache,
    {
        for (i, (xs, y)) in xys_per_line.enumerate() {
            if i == idx.line {
                for (j, x) in xs.enumerate() {
                    if j == idx.idx {
                        return Some((x, y));
                    }
                }
            }
        }
        None
    }


    impl<'a, I, C> Iterator for XysPerLine<'a, I, C>
        where I: Iterator<Item=(&'a str, Rect)>,
              C: CharacterCache,
    {
        // The `Range` occupied by the line across the *y* axis, along with an iterator yielding
        // each possible cursor position along the *x* axis.
        type Item = (Xs<'a, C>, Range);
        fn next(&mut self) -> Option<Self::Item> {
            let XysPerLine { ref mut lines_with_rects, cache, font_size } = *self;
            lines_with_rects.next().map(|(line, line_rect)| {
                let y = line_rect.y;
                let widths = super::char::widths(line.chars(), cache, font_size);
                let xs = Xs {
                    next_x: Some(line_rect.x.start),
                    widths: widths,
                };
                (xs, y)
            })
        }
    }

    impl<'a, C> Iterator for Xs<'a, C>
        where C: CharacterCache,
    {
        // Each possible cursor position along the *x* axis.
        type Item = Scalar;
        fn next(&mut self) -> Option<Self::Item> {
            self.next_x.map(|x| {
                self.next_x = self.widths.next().map(|w| x + w);
                x
            })
        }
    }
}


pub mod line {
    use {Align, CharacterCache, FontSize, GlyphCache, Range, Rect, Scalar};
    use std;

    /// The two types of **Break** indices returned by the **WrapIndicesBy** iterators.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum Break {
        /// The first `usize` is an index at which the string should wrap due to exceeding a
        /// maximum width.
        ///
        /// The second `usize` is the byte length which should be skipped in order to reach the
        /// first non-whitespace character to use as the beginning of the next line.
        Wrap(usize, usize),
        /// An index at which the string breaks due to a newline character, along with the
        /// width of the "newline" token in bytes.
        Newline(usize, usize),
        /// The end of the string has been reached, with the given length.
        End(usize),
    }

    /// Information about a single line of text within a `&str`.
    ///
    /// `Info` is a minimal amount of information that can be stored for efficient reasoning about
    /// blocks of text given some `&str`. The `start` and `end_break` can be used for indexing into
    /// the `&str`, and the `width` can be used for calculating line `Rect`s, alignment, etc.
    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct Info {
        /// The index into the `&str` that represents the first character within the line.
        pub start: usize,
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
    pub struct Infos<'a, C: 'a, F> {
        text: &'a str,
        cache: &'a GlyphCache<C>,
        font_size: FontSize,
        max_width: Scalar,
        next_break_fn: F,
        /// The index that indicates the start of the next line to be yielded.
        start: usize,
    }

    /// An iterator yielding a `Rect` for each line in 
    #[derive(Clone)]
    pub struct Rects<I> {
        infos: I,
        x_align: Align,
        line_spacing: Scalar,
        next: Option<Rect>,
    }

    /// An alias for function pointers that are compatible with the `Block`'s required text
    /// wrapping function.
    pub type NextBreakFnPtr<C> = fn(&str, &GlyphCache<C>, FontSize, Scalar) -> (Break, Scalar);


    impl Break {

        /// Return the index at which the break occurs.
        pub fn index(self) -> usize {
            match self {
                Break::Wrap(idx, _) | Break::Newline(idx, _) | Break::End(idx) => idx,
            }
        }

    }

    impl<'a, C, F> Clone for Infos<'a, C, F>
        where F: Clone,
    {
        fn clone(&self) -> Self {
            Infos {
                text: self.text,
                cache: self.cache,
                font_size: self.font_size,
                max_width: self.max_width,
                next_break_fn: self.next_break_fn.clone(),
                start: self.start,
            }
        }
    }

    impl Info {

        /// The end of the index range for indexing into the slice.
        pub fn end(&self) -> usize {
            self.end_break.index()
        }

        /// The index range for indexing into the original str slice.
        pub fn range(self) -> std::ops::Range<usize> {
            self.start..self.end()
        }

    }

    impl<'a, C> Infos<'a, C, NextBreakFnPtr<C>>
        where C: CharacterCache,
    {

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


    /// Returns the next index at which the text naturally breaks via a newline character,
    /// along with the width of the line.
    fn next_break<C>(text: &str,
                     cache: &GlyphCache<C>,
                     font_size: FontSize) -> (Break, Scalar)
        where C: CharacterCache,
    {
        let mut width = 0.0;
        let mut char_indices = text.char_indices().peekable();
        while let Some((i, ch)) = char_indices.next() {
            // Check for a newline.
            if ch == '\r' {
                if let Some(&(_, '\n')) = char_indices.peek() {
                    return (Break::Newline(i, 2), width)
                }
            } else if ch == '\n' {
                return (Break::Newline(i, 1), width);
            }

            // Update the width.
            width += cache.char_width(font_size, ch);
        }
        (Break::End(text.len()), width)
    }

    /// Returns the next index at which the text will break by either:
    /// - A newline character.
    /// - A line wrap at the beginning of the first character exceeding the `max_width`.
    ///
    /// Also returns the width of each line alongside the Break.
    fn next_break_by_character<C>(text: &str,
                                  cache: &GlyphCache<C>,
                                  font_size: FontSize,
                                  max_width: Scalar) -> (Break, Scalar)
        where C: CharacterCache,
    {
        let mut width = 0.0;
        let mut char_indices = text.char_indices().peekable();
        while let Some((i, ch)) = char_indices.next() {

            // Check for a newline.
            if ch == '\r' {
                if let Some(&(_, '\n')) = char_indices.peek() {
                    return (Break::Newline(i, 2), width)
                }
            } else if ch == '\n' {
                return (Break::Newline(i, 1), width);
            }

            // Add the character's width to the width so far.
            let new_width = width + cache.char_width(font_size, ch);

            // Check for a line wrap.
            if new_width > max_width {
                return (Break::Wrap(i, 0), width);
            }

            width = new_width;
        }

        (Break::End(text.len()), width)
    }

    /// Returns the next index at which the text will break by either:
    /// - A newline character.
    /// - A line wrap at the beginning of the whitespace that preceeds the first word
    /// exceeding the `max_width`.
    ///
    /// Also returns the width the line alongside the Break.
    fn next_break_by_whitespace<C>(text: &str,
                                   cache: &GlyphCache<C>,
                                   font_size: FontSize,
                                   max_width: Scalar) -> (Break, Scalar)
        where C: CharacterCache,
    {
        let mut width = 0.0;
        let mut last_whitespace_start = (0, 0.0);
        let mut char_indices = text.char_indices().peekable();
        while let Some((i, ch)) = char_indices.next() {

            // Check for a newline.
            if ch == '\r' {
                if let Some(&(_, '\n')) = char_indices.peek() {
                    return (Break::Newline(i, 2), width)
                }
            } else if ch == '\n' {
                return (Break::Newline(i, 1), width);
            }

            // Check for a new whitespace.
            else if ch.is_whitespace() {
                last_whitespace_start = (i, width);
            }

            // Add the character's width to the width so far.
            let new_width = width + cache.char_width(font_size, ch);

            // Check for a line wrap.
            if width > max_width {
                let (start_idx, width_before_whitespace) = last_whitespace_start;
                return (Break::Wrap(start_idx, 1), width_before_whitespace);
            }

            width = new_width;
        }

        (Break::End(text.len()), width)
    }


    /// Produce an `Infos` iterator wrapped by the given `next_break_fn`.
    pub fn infos_wrapped_by<'a, C, F>(text: &'a str,
                                      cache: &'a GlyphCache<C>,
                                      font_size: FontSize,
                                      max_width: Scalar,
                                      next_break_fn: F) -> Infos<'a, C, F>
        where F: for<'b> FnMut(&'b str, &'b GlyphCache<C>, FontSize, Scalar) -> (Break, Scalar)
    {
        Infos {
            text: text,
            cache: cache,
            font_size: font_size,
            max_width: max_width,
            next_break_fn: next_break_fn,
            start: 0,
        }
    }

    /// Produce an `Infos` iterator that yields an `Info` for every line in the given text.
    ///
    /// The produced `Infos` iterator will not wrap the text, and only break each line via newline
    /// characters within the text (either `\n` or `\r\n`).
    pub fn infos<'a, C>(text: &'a str,
                        cache: &'a GlyphCache<C>,
                        font_size: FontSize) -> Infos<'a, C, NextBreakFnPtr<C>>
        where C: CharacterCache,
    {
        fn no_wrap<C>(text: &str,
                      cache: &GlyphCache<C>,
                      font_size: FontSize,
                      _max_width: Scalar) -> (Break, Scalar)
            where C: CharacterCache,
        {
            next_break(text, cache, font_size)
        }

        infos_wrapped_by(text, cache, font_size, std::f64::MAX, no_wrap)
    }

    /// Produce an iterator yielding the bounding `Rect` for each line in the text.
    ///
    /// This function assumes that `font_size` is the same `FontSize` used to produce the `Info`s
    /// yielded by the `infos` Iterator.
    pub fn rects<I>(mut infos: I,
                    font_size: FontSize,
                    bounding_rect: Rect,
                    x_align: Align,
                    y_align: Align,
                    line_spacing: Scalar) -> Rects<I>
        where I: Iterator<Item=Info> + ExactSizeIterator,
    {
        let first_rect = infos.next().map(|first_info| {

            // Calculate the `x` `Range` of the first line `Rect`.
            let range = Range::new(0.0, first_info.width);
            let x = match x_align {
                Align::Start => range.align_start_of(bounding_rect.x),
                Align::Middle => range.align_middle_of(bounding_rect.x),
                Align::End => range.align_end_of(bounding_rect.x),
            };

            // Calculate the `y` `Range` of the first line `Rect`.
            let num_lines = infos.len();
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


    impl<'a, C, F> Iterator for Infos<'a, C, F>
        where C: CharacterCache,
              F: for<'b> FnMut(&'b str, &'b GlyphCache<C>, FontSize, Scalar) -> (Break, Scalar)
    {
        type Item = Info;
        fn next(&mut self) -> Option<Self::Item> {
            let Infos {
                text,
                cache,
                font_size,
                max_width,
                ref mut next_break_fn,
                ref mut start
            } = *self;

            match next_break_fn(&text[*start..], cache, font_size, max_width) {
                (next @ Break::Newline(_, _), width) | (next @ Break::Wrap(_, _), width) => {
                    let next = match next {
                        Break::Newline(idx, width) => Break::Newline(*start + idx, width),
                        Break::Wrap(idx, width) => Break::Wrap(*start + idx, width),
                        _ => unreachable!(),
                    };
                    let info = Info {
                        start: *start,
                        end_break: next,
                        width: width,
                    };
                    *start = match next {
                        Break::Newline(idx, width) => idx + width,
                        Break::Wrap(idx, width) => idx + width,
                        _ => unreachable!(),
                    };
                    Some(info)
                },
                (Break::End(_), width) =>
                    if *start < text.len() {
                        let info = Info {
                            start: *start,
                            end_break: Break::End(text.len()),
                            width: width,
                        };
                        *start = text.len();
                        Some(info)
                    } else {
                        None
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
                            Align::Start => range.align_start_of(line_rect.x),
                            Align::Middle => range.align_middle_of(line_rect.x),
                            Align::End => range.align_end_of(line_rect.x),
                        }
                    };

                    Rect { x: x, y: y }
                });

                line_rect
            })
        }
    }

}
