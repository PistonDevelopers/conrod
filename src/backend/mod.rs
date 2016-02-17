//! Traits and functionality related to Conrod's generic backend.
//!
//! These modules describe an interface allowing users to use their own graphics, events and
//! character caching systems with Conrod.
//!
//! **Note:** Conrod currently heavily depends upon the piston graphics and event crates for
//! enabling genericity over custom user backends. This dependency may change in the near future in
//! favour of simplified conrod-specific backend trait.

pub use self::graphics::{CharacterCache, Graphics};

pub mod graphics;

/// A trait to be implemented by all backends to conrod.
///
/// This trait allows conrod to remain entirely backend agnostic so that users may use conrod with
/// any window, graphics or font contexts.
pub trait Backend {
    /// The `Texture` type used by the `Graphics` and `CharacterCache` backends.
    type Texture: self::graphics::ImageSize;
    /// The part of the backend used for handling graphics.
    ///
    /// `Graphics` backends must implement the [`Graphics` trait](./trait.Graphics.html).
    type Graphics: Graphics<Texture=Self::Texture>;
    /// The character cache used by the backend.
    ///
    /// Must implement the [`CharacterCache` trait](./trait.CharacterCache.html).
    type CharacterCache: CharacterCache<Texture=Self::Texture>;
}
