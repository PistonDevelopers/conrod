//! Conrod's character caching API.
//!
//! Provides types and functionality related character texture caching and text dimensioning.

use {CharacterCache, FontSize, Scalar};
use std::cell::RefCell;


/// A wrapper over some CharacterCache, exposing it's functionality via a RefCell.
///
/// The GlyphCache is frequently needed in order to calculate text dimensions. We wrap the
/// CharacterCache in a RefCell in order to avoid ownership issues that this may cause.
pub struct GlyphCache<C> {
    ref_cell: RefCell<C>,
}

/// An iterator yielding the widths of each consecutive character in some sequence.
#[derive(Clone)]
pub struct CharWidths<'a, C: 'a, I> {
    font_size: FontSize,
    cache: &'a GlyphCache<C>,
    chars: I,
}

/// Some position along the X axis (used within `CharXs`).
pub type X = Scalar;

/// The half of the width of some character (used within `CharXs`).
pub type HalfW = Scalar;

/// An iterator that converts the given sequnce of `char`s into their consecutive positions along
/// the x-axis.
#[derive(Clone)]
pub struct CharXs<'a, C: 'a, I> {
    widths: CharWidths<'a, C, I>,
    maybe_next: Option<(HalfW, X)>,
}

/// An iterator yielding the widths of each consecutive `&str` in some sequence.
#[derive(Clone)]
pub struct Widths<'a, C: 'a, I> {
    font_size: FontSize,
    cache: &'a GlyphCache<C>,
    strs: I,
}


/// The two types of **LineBreak** indices returned by the **WrapIndicesBy** iterators.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LineBreak {
    /// The first `usize` is an index at which the string should wrap due to exceeding a maximum
    /// width.
    ///
    /// The second `usize` is the byte length which should be skipped in order to reach the first
    /// non-whitespace character to use as the beginning of the next line.
    Wrap(usize, usize),
    /// An index at which the string breaks due to a newline character, along with the width of the
    /// "newline" token in bytes.
    Newline(usize, usize),
}

/// An iterator that yields the indices at which some text should wrap in accordance with the given
/// wrap function.
#[derive(Clone)]
pub struct LineBreaksBy<'a, C: 'a, F> {
    font_size: FontSize,
    cache: &'a GlyphCache<C>,
    text: &'a str,
    max_width: Scalar,
    line_break_fn: F,
    start: usize,
}

/// A function that returns the first index at which the text should wrap for the given max width.
pub type NextLineBreakFn<C> = fn(&GlyphCache<C>, FontSize, &str, Scalar) -> Option<LineBreak>;

/// An iterator that yields the indices at which some text should wrap via a character.
pub type LineBreaksByCharacter<'a, C> = LineBreaksBy<'a, C, NextLineBreakFn<C>>;

/// An iterator that yields the indices at which some text should wrap via whitespace.
pub type LineBreaksByWhitespace<'a, C> = LineBreaksBy<'a, C, NextLineBreakFn<C>>;

/// A wrapper over an iterator yielding **LineBreak**s that yields each line divided by the breaks.
pub struct Lines<'a, I> {
    text: &'a str,
    line_breaks: I,
}

/// An iterator yielding lines for text wrapped with the given function.
pub type LinesWrappedBy<'a, C, F> = Lines<'a, LineBreaksBy<'a, C, F>>;

/// An iterator yielding lines for text wrapped via the first character exceeding a max width.
pub type LinesWrappedByCharacter<'a, C> = LinesWrappedBy<'a, C, NextLineBreakFn<C>>;

/// An iterator yielding lines for text wrapped via the first character exceeding a max width.
pub type LinesWrappedByWhitespace<'a, C> = LinesWrappedBy<'a, C, NextLineBreakFn<C>>;


impl<C> GlyphCache<C> {

    /// Construct a new **GlyphCache**.
    pub fn new(cache: C) -> Self {
        GlyphCache {
            ref_cell: RefCell::new(cache),
        }
    }

    /// The width of a single character with the given size.
    pub fn char_width(&self, font_size: FontSize, ch: char) -> Scalar
        where C: CharacterCache,
    {
        self.ref_cell.borrow_mut().character(font_size, ch).width()
    }

    /// Converts the given sequence of `char`s into their Scalar widths.
    pub fn char_widths<I>(&self, font_size: FontSize, chars: I) -> CharWidths<C, I::IntoIter>
        where I: IntoIterator<Item=char>,
    {
        CharWidths {
            font_size: font_size,
            cache: self,
            chars: chars.into_iter(),
        }
    }

    /// Converts the given sequnce of `char`s into their consecutive positions along the x-axis.
    pub fn char_xs<I>(&self, font_size: FontSize, start_x: Scalar, chars: I) -> CharXs<C, I::IntoIter>
        where C: CharacterCache,
              I: IntoIterator<Item=char>,
    {
        let mut widths = self.char_widths(font_size, chars);
        let maybe_first = widths.next().map(|w| (w / 2.0, start_x));
        CharXs {
            maybe_next: maybe_first,
            widths: widths,
        }
    }

    /// Return the width of the given text.
    pub fn width(&self, font_size: FontSize, text: &str) -> Scalar
        where C: CharacterCache,
    {
        self.ref_cell.borrow_mut().width(font_size, text)
    }

    /// Converts the given sequence of `&str`s into their Scalar widths.
    pub fn widths<I>(&self, font_size: FontSize, strs: I) -> Widths<C, I::IntoIter>
        where for<'a> I: IntoIterator<Item=&'a str>,
    {
        Widths {
            font_size: font_size,
            cache: self,
            strs: strs.into_iter(),
        }
    }

    /// An iterator that yields the indices at which some text should wrap in accordance with the
    /// given wrap function.
    pub fn line_breaks_by<'a, F>(&'a self,
                                 font_size: FontSize,
                                 text: &'a str,
                                 max_width: Scalar,
                                 line_break_fn: F) -> LineBreaksBy<'a, C, F>
    {
        LineBreaksBy {
            font_size: font_size,
            cache: self,
            text: text,
            start: 0,
            max_width: max_width,
            line_break_fn: line_break_fn,
        }
    }

    /// An iterator that yields the indices at which some text should wrap via a character.
    pub fn line_breaks_by_character<'a>(&'a self,
                                        font_size: FontSize,
                                        text: &'a str,
                                        max_width: Scalar) -> LineBreaksByCharacter<'a, C>
        where C: CharacterCache,
    {
        self.line_breaks_by(font_size, text, max_width, LineBreak::next_by_character)
    }

    /// An iterator that yields the indices at which some text should wrap via whitespace.
    pub fn line_breaks_by_whitespace<'a>(&'a self,
                                         font_size: FontSize,
                                         text: &'a str,
                                         max_width: Scalar) -> LineBreaksByWhitespace<'a, C>
        where C: CharacterCache,
    {
        self.line_breaks_by(font_size, text, max_width, LineBreak::next_by_whitespace)
    }

    /// An iterator that behaves the same as `text.lines()` but inserts a break before the first
    /// character that would cause the line to exceed the given `max_width`.
    pub fn lines_wrapped_by<'a, F>(&'a self,
                                   font_size: FontSize,
                                   text: &'a str,
                                   max_width: Scalar,
                                   wrap_fn: F) -> LinesWrappedBy<'a, C, F>
    {
        let line_breaks = self.line_breaks_by(font_size, text, max_width, wrap_fn);
        Lines::new(text, line_breaks)
    }

    /// An iterator that behaves the same as `text.lines()` but inserts a break before the first
    /// character that would cause the line to exceed the given `max_width`.
    pub fn lines_wrapped_by_character<'a>(&'a self,
                                          font_size: FontSize,
                                          text: &'a str,
                                          max_width: Scalar) -> LinesWrappedByCharacter<'a, C>
        where C: CharacterCache,
    {
        let line_breaks = self.line_breaks_by_character(font_size, text, max_width);
        Lines::new(text, line_breaks)
    }

    /// An iterator that behaves the same as `text.lines()` but inserts a break before the first
    /// character that would cause the line to exceed the given `max_width`.
    pub fn lines_wrapped_by_whitespace<'a>(&'a self,
                                           font_size: FontSize,
                                           text: &'a str,
                                           max_width: Scalar) -> LinesWrappedByWhitespace<'a, C>
        where C: CharacterCache,
    {
        let line_breaks = self.line_breaks_by_character(font_size, text, max_width);
        Lines::new(text, line_breaks)
    }

}


impl<C> ::std::ops::Deref for GlyphCache<C> {
    type Target = RefCell<C>;
    fn deref<'a>(&'a self) -> &'a RefCell<C> {
        &self.ref_cell
    }
}

impl<C> ::std::ops::DerefMut for GlyphCache<C> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut RefCell<C> {
        &mut self.ref_cell
    }
}


impl<'a, C, I> Iterator for CharWidths<'a, C, I>
    where C: CharacterCache,
          I: Iterator<Item=char>,
{
    type Item = Scalar;
    fn next(&mut self) -> Option<Self::Item> {
        let CharWidths { font_size, cache, ref mut chars } = *self;
        chars.next().map(|ch| cache.char_width(font_size, ch))
    }
}

impl<'a, C, I> Iterator for Widths<'a, C, I>
    where C: CharacterCache,
          for<'b> I: Iterator<Item=&'b str>,
{
    type Item = Scalar;
    fn next(&mut self) -> Option<Self::Item> {
        let Widths { font_size, cache, ref mut strs } = *self;
        strs.next().map(|s| cache.width(font_size, s))
    }
}

impl<'a, C, I> Iterator for CharXs<'a, C, I>
    where C: CharacterCache,
          I: Iterator<Item=char>,
{
    type Item = X;
    fn next(&mut self) -> Option<Self::Item> {
        self.maybe_next.take().map(|(half_w, x)| {
            self.maybe_next = self.widths.next().map(|next_w| {
                let next_half_w = next_w / 2.0;
                let step = half_w + next_half_w;
                (next_half_w, x + step)
            });
            x
        })
    }
}

impl<'a, C, F> Iterator for LineBreaksBy<'a, C, F>
    where C: CharacterCache,
          for<'b> F: FnMut(&'b GlyphCache<C>, FontSize, &'b str, Scalar) -> Option<LineBreak>,
{
    /// The index into the start of the line along with the line's break if it has one.
    type Item = (usize, Option<LineBreak>);
    fn next(&mut self) -> Option<Self::Item> {
        let LineBreaksBy {
            cache,
            font_size,
            ref text,
            max_width,
            ref mut line_break_fn,
            ref mut start,
        } = *self;

        match line_break_fn(cache, font_size, &text[*start..], max_width) {
            Some(next) => {
                let next = match next {
                    LineBreak::Newline(idx, width) => LineBreak::Newline(*start + idx, width),
                    LineBreak::Wrap(idx, width) => LineBreak::Wrap(*start + idx, width),
                };
                let range = (*start, Some(next));
                *start = match next {
                    LineBreak::Newline(idx, width) => idx + width,
                    LineBreak::Wrap(idx, width) => idx + width,
                };
                Some(range)
            },
            None => if *start < text.len() {
                let last = Some((*start, None));
                *start = text.len();
                last
            } else {
                None
            },
        }
    }
}

impl<'a, I> Iterator for Lines<'a, I>
    where I: Iterator<Item=(usize, Option<LineBreak>)>,
{
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let Lines {
            ref text,
            ref mut line_breaks,
        } = *self;
        line_breaks.next().map(|(start, maybe_line_break)| match maybe_line_break {
            Some(line_break) => &text[start..line_break.index()],
            None => &text[start..],
        })
    }
}


impl LineBreak {

    /// Extracts the index at which the break occurs within the text (i.e. the index following the
    /// last byte of the line).
    pub fn index(self) -> usize {
        match self {
            LineBreak::Wrap(idx, _) | LineBreak::Newline(idx, _) => idx,
        }
    }

    /// Returns the next index at which the text will break by either:
    /// - A newline character.
    /// - A line wrap at the beginning of the first character exceeding the `max_width`.
    pub fn next_by_character<C>(cache: &GlyphCache<C>,
                                font_size: FontSize,
                                text: &str,
                                max_width: Scalar) -> Option<Self>
        where C: CharacterCache,
    {
        let mut width = 0.0;
        let mut char_indices = text.char_indices().peekable();
        while let Some((i, ch)) = char_indices.next() {

            // Check for a newline.
            if ch == '\r' {
                if let Some(&(_, '\n')) = char_indices.peek() {
                    return Some(LineBreak::Newline(i, 2))
                }
            } else if ch == '\n' {
                return Some(LineBreak::Newline(i, 1));
            }

            // Update the width.
            width += cache.char_width(font_size, ch);

            // Check for a line wrap.
            if width > max_width {
                return Some(LineBreak::Wrap(i, 0));
            }
        }
        None
    }

    /// Returns the next index at which the text will break by either:
    /// - A newline character.
    /// - A line wrap at the beginning of the whitespace that preceeds the first word exceeding the
    /// `max_width`.
    pub fn next_by_whitespace<C>(cache: &GlyphCache<C>,
                                 font_size: FontSize,
                                 text: &str,
                                 max_width: Scalar) -> Option<Self>
        where C: CharacterCache,
    {
        let mut width = 0.0;
        let mut last_whitespace_start = 0;
        let mut char_indices = text.char_indices().peekable();
        while let Some((i, ch)) = char_indices.next() {

            // Check for a newline.
            if ch == '\r' {
                if let Some(&(_, '\n')) = char_indices.peek() {
                    return Some(LineBreak::Newline(i, 2))
                }
            } else if ch == '\n' {
                return Some(LineBreak::Newline(i, 1));
            }

            // Check for a new whitespace.
            else if ch.is_whitespace() {
                last_whitespace_start = i;
            }

            // Update the width.
            width += cache.char_width(font_size, ch);

            // Check for a line wrap.
            if width > max_width {
                return Some(LineBreak::Wrap(last_whitespace_start, 1));
            }
        }
        None
    }

}


impl<'a, I> Lines<'a, I> {
    /// Construct a new **Lines** iterator from the given text and line_breaks.
    ///
    /// **Note:** This function assumes that the given `line_breaks` correctly represent all line
    /// breaks within the given `text`, starting from the `0`th byte index.
    pub fn new(text: &'a str, line_breaks: I) -> Self {
        Lines {
            text: text,
            line_breaks: line_breaks,
        }
    }
}
