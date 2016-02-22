//! And implementation of conrod's `Backend` trait for the piston_window crate.
//!
//! This module can be enabled by passing the "backend-piston_window" feature to cargo when
//! building conrod.

extern crate piston_window;

/// A type upon which we will implement the `Backend` trait for the `piston_window` crate.
pub struct Backend<'a>(::std::marker::PhantomData<&'a ()>);

impl<'a> super::Backend for Backend<'a> {
    type Texture = <piston_window::G2d<'a> as super::Graphics>::Texture;
    type CharacterCache = piston_window::Glyphs;
}

/// An alias for the `Ui` type compatible with our `piston_window` backend.
pub type Ui = ::Ui<Backend<'static>>;
/// An alias for the `UiCell` type compatible with our `piston_window` backend.
pub type UiCell<'a> = ::UiCell<'a, Backend<'static>>;
