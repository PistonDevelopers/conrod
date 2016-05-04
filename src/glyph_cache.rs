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

    /// Return the width of the given text.
    pub fn width(&self, font_size: FontSize, text: &str) -> Scalar
        where C: CharacterCache,
    {
        self.ref_cell.borrow_mut().width(font_size, text)
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
