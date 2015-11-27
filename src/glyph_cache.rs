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

/// An iterator yielding the widths of each consecutive `&str` in some sequence.
#[derive(Clone)]
pub struct Widths<'a, C: 'a, I> {
    font_size: FontSize,
    cache: &'a GlyphCache<C>,
    strs: I,
}

/// An iterator that behaves the same as `text.lines()` but inserts breaks on each line where
/// specifid by the given wrapping function `F`.
#[derive(Clone)]
pub struct WrapBy<'a, C: 'a, F> {
    font_size: FontSize,
    cache: &'a GlyphCache<C>,
    lines: ::std::str::Lines<'a>,
    maybe_unfinished_line: Option<&'a str>,
    max_width: Scalar,
    wrap_fn: F,
}

/// An iterator that behaves the same as `text.lines()` but inserts a break before the first
/// **character** that would cause the line to exceed the given `max_width`.
pub type WrapByCharacter<'a, C> =
    WrapBy<'a, C, fn(&GlyphCache<C>, FontSize, &str, Scalar) -> Option<usize>>;

/// An iterator that behaves the same as `text.lines()` but inserts a break before the whitespace
/// prior to the first **word** that would cause the line to exceed the given `max_width`.
pub type WrapByWhitespace<'a, C> =
    WrapBy<'a, C, fn(&GlyphCache<C>, FontSize, &str, Scalar) -> Option<usize>>;


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

    /// Return the width of the given text.
    pub fn width(&self, font_size: FontSize, text: &str) -> Scalar
        where C: CharacterCache,
    {
        self.ref_cell.borrow_mut().width(font_size, text)
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

    /// An iterator that behaves the same as `text.lines()` but inserts a break before the first
    /// character that would cause the line to exceed the given `max_width`.
    pub fn wrap_by<'a, F>(&'a self,
                          font_size: FontSize,
                          text: &'a str,
                          max_width: Scalar,
                          wrap_fn: F) -> WrapBy<'a, C, F>
    {
        WrapBy {
            font_size: font_size,
            cache: self,
            lines: text.lines(),
            maybe_unfinished_line: None,
            max_width: max_width,
            wrap_fn: wrap_fn,
        }
    }

    /// An iterator that behaves the same as `text.lines()` but inserts a break before the first
    /// character that would cause the line to exceed the given `max_width`.
    pub fn wrap_by_character<'a>(&'a self,
                                 font_size: FontSize,
                                 text: &'a str,
                                 max_width: Scalar) -> WrapByCharacter<'a, C>
        where C: CharacterCache,
    {
        self.wrap_by(font_size, text, max_width, wrap_by_character)
    }

    /// An iterator that behaves the same as `text.lines()` but inserts a break before the first
    /// character that would cause the line to exceed the given `max_width`.
    pub fn wrap_by_whitespace<'a>(&'a self,
                                  font_size: FontSize,
                                  text: &'a str,
                                  max_width: Scalar) -> WrapByWhitespace<'a, C>
        where C: CharacterCache,
    {
        self.wrap_by(font_size, text, max_width, wrap_by_whitespace)
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

impl<'a, C, F> Iterator for WrapBy<'a, C, F>
    where C: CharacterCache,
          for<'b> F: FnMut(&'b GlyphCache<C>, FontSize, &'b str, Scalar) -> Option<usize>
{
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let WrapBy {
            font_size,
            cache,
            ref mut lines,
            ref mut maybe_unfinished_line,
            max_width,
            ref mut wrap_fn,
        } = *self;

        // Returns the first index at which the line should wrap.
        let mut wrap = |line: &str| wrap_fn(cache, font_size, line, max_width);

        match maybe_unfinished_line.take() {
            Some(line) => match wrap(line) {
                Some(i) => {
                    let (front, back) = line.split_at(i);
                    *maybe_unfinished_line = Some(back);
                    Some(front)
                },
                None => Some(line),
            },
            None => lines.next().map(|line| match wrap(line) {
                Some(i) => {
                    let (front, back) = line.split_at(i);
                    *maybe_unfinished_line = Some(back);
                    front
                },
                None => line,
            }),
        }
    }
}


fn wrap_by_character<C>(cache: &GlyphCache<C>,
                        font_size: FontSize,
                        line: &str,
                        max_width: Scalar) -> Option<usize>
    where C: CharacterCache,
{
    let mut width = 0.0;
    for (i, ch) in line.char_indices() {
        width += cache.char_width(font_size, ch);
        if width > max_width {
            return Some(i);
        }
    }
    None
}

fn wrap_by_whitespace<C>(cache: &GlyphCache<C>,
                         font_size: FontSize,
                         line: &str,
                         max_width: Scalar) -> Option<usize>
    where C: CharacterCache,
{
    let mut width = 0.0;
    let mut last_whitespace_start = 0;
    for (i, ch) in line.char_indices() {
        if ch.is_whitespace() {
            last_whitespace_start = i;
        }
        width += cache.char_width(font_size, ch);
        if width > max_width {
            return Some(last_whitespace_start);
        }
    }
    None
}

