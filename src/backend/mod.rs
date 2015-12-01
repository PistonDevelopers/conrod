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
