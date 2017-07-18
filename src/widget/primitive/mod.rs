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

use {Point, Range, Rect};

/// Find the bounding rect for the given series of points.
pub fn bounding_box_for_points<I>(mut points: I) -> Rect
    where I: Iterator<Item=Point>,
{
    points.next().map(|first| {
        let start_rect = Rect {
            x: Range { start: first[0], end: first[0] },
            y: Range { start: first[1], end: first[1] },
        };
        points.fold(start_rect, Rect::stretch_to_point)
    }).unwrap_or_else(|| Rect::from_xy_dim([0.0, 0.0], [0.0, 0.0]))
}
