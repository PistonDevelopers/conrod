//! Traits and functionality related to Conrod's generic backend.
//!
//! These modules describe an interface allowing users to use their own graphics, events and
//! character caching systems with Conrod.
//!
//! **Note:** Conrod currently heavily depends upon the piston graphics and event crates for
//! enabling genericity over custom user backends. This dependency may change in the near future in
//! favour of simplified conrod-specific backend trait.

pub use self::graphics::{CharacterCache, Graphics};
use std;

pub mod graphics;

/// A trait to be implemented by all backends to conrod.
///
/// This trait allows conrod to remain entirely backend agnostic so that users may use conrod with
/// any window, graphics or font contexts.
///
/// Conrod provides a blanket implementation for all `(T, C)` tuples, where `T` is some texture and
/// `C` is some character cache and both satisfy the necessary bounds.
pub trait Backend {
    /// The `Texture` type used by the `Graphics` and `CharacterCache` backends.
    type Texture: self::graphics::ImageSize + std::any::Any;
    /// The character cache used by the backend.
    ///
    /// Must implement the [`CharacterCache` trait](./trait.CharacterCache.html).
    type CharacterCache: CharacterCache<Texture=Self::Texture>;
}


impl<T, C> Backend for (T, C)
    where T: self::graphics::ImageSize + std::any::Any,
          C: CharacterCache<Texture=T>,
{
    type Texture = T;
    type CharacterCache = C;
}
