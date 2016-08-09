//! Primitive widgets are special in that they are built into conrod's `render`ing logic.
//!
//! By providing a set of foundational graphics widgets, we avoid the need for other widgets to
//! define their own methods for rendering. Instead, conrod graphics backends only need to define
//! rendering methods for a small set of primitives.

pub mod line;
pub mod image;
pub mod point_path;
pub mod shape;
pub mod text;
