//! Conrod's character caching API.
//!
//! Provides types and functionality related character texture caching and text dimensioning.

use {CharacterCache, FontSize, Scalar};
use std::cell::RefCell;
use text;


/// A wrapper over some CharacterCache, exposing it's functionality via a RefCell.
///
/// The GlyphCache is frequently needed in order to calculate text dimensions. We wrap the
/// CharacterCache in a RefCell in order to avoid ownership issues that this may cause.
pub struct GlyphCache<C> {
    ref_cell: RefCell<C>,
}


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
    pub fn char_widths<I>(&self,
                          font_size: FontSize,
                          chars: I) -> text::char::Widths<I::IntoIter, C>
        where I: IntoIterator<Item=char>,
              C: CharacterCache,
    {
        text::char::widths(chars, self, font_size)
    }

    /// Converts the given sequnce of `char`s into their consecutive positions along the x-axis.
    ///
    /// `start_x` represents the first character's `x` position.
    pub fn char_xs<I>(&self, font_size: FontSize, chars: I) -> text::char::Xs<I::IntoIter, C>
        where C: CharacterCache,
              I: IntoIterator<Item=char>,
    {
        text::char::xs(chars, self, font_size)
    }

    /// Return the width of the given text.
    pub fn width(&self, font_size: FontSize, text: &str) -> Scalar
        where C: CharacterCache,
    {
        self.ref_cell.borrow_mut().width(font_size, text)
    }

    /// Converts the given sequence of `&str`s into their Scalar widths.
    pub fn widths<I>(&self, font_size: FontSize, strs: I) -> text::str::Widths<I::IntoIter, C>
        where for<'a> I: IntoIterator<Item=&'a str>,
              C: CharacterCache,
    {
        text::str::widths(strs, self, font_size)
    }

    /// An iterator that yields the indices at which some text should wrap in accordance with the
    /// given wrap function.
    pub fn line_breaks_by<'a, F>(&'a self,
                                 font_size: FontSize,
                                 text: &'a str,
                                 max_width: Scalar,
                                 line_break_fn: F) -> text::str::line::BreaksBy<'a, C, F>
    {
        text::str::line::breaks_by(text, self, font_size, max_width, line_break_fn)
    }

    /// An iterator that yields the indices at which some text should wrap via a character.
    pub fn line_breaks_by_character<'a>(&'a self,
                                        font_size: FontSize,
                                        text: &'a str,
                                        max_width: Scalar) -> text::str::line::BreaksByCharacter<'a, C>
        where C: CharacterCache,
    {
        text::str::line::breaks_by_character(text, self, font_size, max_width)
    }

    /// An iterator that yields the indices at which some text should wrap via whitespace.
    pub fn line_breaks_by_whitespace<'a>(&'a self,
                                         font_size: FontSize,
                                         text: &'a str,
                                         max_width: Scalar) -> text::str::line::BreaksByWhitespace<'a, C>
        where C: CharacterCache,
    {
        text::str::line::breaks_by_whitespace(text, self, font_size, max_width)
    }

    /// An iterator that behaves the same as `text.lines()` but inserts a break before the first
    /// character that would cause the line to exceed the given `max_width`.
    pub fn lines_wrapped_by<'a, F>(&'a self,
                                   font_size: FontSize,
                                   text: &'a str,
                                   max_width: Scalar,
                                   wrap_fn: F) -> text::str::line::WrappedBy<'a, C, F>
    {
        text::str::line::wrapped_by(text, self, font_size, max_width, wrap_fn)
    }

    /// An iterator that behaves the same as `text.lines()` but inserts a break before the first
    /// character that would cause the line to exceed the given `max_width`.
    pub fn lines_wrapped_by_character<'a>(&'a self,
                                          font_size: FontSize,
                                          text: &'a str,
                                          max_width: Scalar) -> text::str::line::WrappedByCharacter<'a, C>
        where C: CharacterCache,
    {
        text::str::line::wrapped_by_character(text, self, font_size, max_width)
    }

    /// An iterator that behaves the same as `text.lines()` but inserts a break before the first
    /// character that would cause the line to exceed the given `max_width`.
    pub fn lines_wrapped_by_whitespace<'a>(&'a self,
                                           font_size: FontSize,
                                           text: &'a str,
                                           max_width: Scalar) -> text::str::line::WrappedByWhitespace<'a, C>
        where C: CharacterCache,
    {
        text::str::line::wrapped_by_whitespace(text, self, font_size, max_width)
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
