//! Defines the scalar `Rect` type used throughout conrod.

use super::{Dimensions, Padding, Point, Range, Scalar};


/// Defines a Rectangle's bounds across the x and y axes.
///
/// This is a conrod-specific Rectangle in that it's designed to help with layout.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rect {
    /// The start and end positions of the Rectangle on the x axis.
    pub x: Range,
    /// The start and end positions of the Rectangle on the y axis.
    pub y: Range,
}

/// Either of the four corners of a **Rect**.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Corner {
    /// The top left corner of a **Rect**.
    TopLeft,
    /// The top right corner of a **Rect**.
    TopRight,
    /// The bottom left corner of a **Rect**.
    BottomLeft,
    /// The bottom right corner of a **Rect**.
    BottomRight,
}


impl Rect {

    /// Construct a Rect from a given `Point` and `Dimensions`.
    pub fn from_xy_dim(xy: Point, dim: Dimensions) -> Self {
        Rect {
            x: Range::from_pos_and_len(xy[0], dim[0]),
            y: Range::from_pos_and_len(xy[1], dim[1]),
        }
    }

    /// Construct a Rect from the coordinates of two points.
    pub fn from_corners(a: Point, b: Point) -> Self {
        let (left, right) = if a[0] < b[0] { (a[0], b[0]) } else { (b[0], a[0]) };
        let (bottom, top) = if a[1] < b[1] { (a[1], b[1]) } else { (b[1], a[1]) };
        Rect {
            x: Range { start: left, end: right },
            y: Range { start: bottom, end: top },
        }
    }

    /// The Rect representing the area in which two Rects overlap.
    pub fn overlap(self, other: Self) -> Option<Self> {
        self.x.overlap(other.x).and_then(|x| self.y.overlap(other.y).map(|y| Rect { x: x, y: y }))
    }

    /// The Rect that encompass the two given sets of Rect.
    pub fn max(self, other: Self) -> Self {
        Rect {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    /// The position in the middle of the x bounds.
    pub fn x(&self) -> Scalar {
        self.x.middle()
    }

    /// The position in the middle of the y bounds.
    pub fn y(&self) -> Scalar {
        self.y.middle()
    }

    /// The xy position in the middle of the bounds.
    pub fn xy(&self) -> Point {
        [self.x(), self.y()]
    }

    /// The centered x and y coordinates as a tuple.
    pub fn x_y(&self) -> (Scalar, Scalar) {
        (self.x(), self.y())
    }

    /// The width of the Rect.
    pub fn w(&self) -> Scalar {
        self.x.len()
    }

    /// The height of the Rect.
    pub fn h(&self) -> Scalar {
        self.y.len()
    }

    /// The total dimensions of the Rect.
    pub fn dim(&self) -> Dimensions {
        [self.w(), self.h()]
    }

    /// The width and height of the Rect as a tuple.
    pub fn w_h(&self) -> (Scalar, Scalar) {
        (self.w(), self.h())
    }

    /// Convert the Rect to a `Point` and `Dimensions`.
    pub fn xy_dim(&self) -> (Point, Dimensions) {
        (self.xy(), self.dim())
    }

    /// The Rect's centered coordinates and dimensions in a tuple.
    pub fn x_y_w_h(&self) -> (Scalar, Scalar, Scalar, Scalar) {
        let (xy, dim) = self.xy_dim();
        (xy[0], xy[1], dim[0], dim[1])
    }

    /// The length of the longest side of the rectangle.
    pub fn len(&self) -> Scalar {
        ::utils::partial_max(self.w(), self.h())
    }

    /// The Rect's lowest y value.
    pub fn bottom(&self) -> Scalar {
        self.y.undirected().start
    }

    /// The Rect's highest y value.
    pub fn top(&self) -> Scalar {
        self.y.undirected().end
    }

    /// The Rect's lowest x value.
    pub fn left(&self) -> Scalar {
        self.x.undirected().start
    }

    /// The Rect's highest x value.
    pub fn right(&self) -> Scalar {
        self.x.undirected().end
    }

    /// The top left corner **Point**.
    pub fn top_left(&self) -> Point {
        [self.left(), self.top()]
    }

    /// The bottom left corner **Point**.
    pub fn bottom_left(&self) -> Point {
        [self.left(), self.bottom()]
    }

    /// The top right corner **Point**.
    pub fn top_right(&self) -> Point {
        [self.right(), self.top()]
    }

    /// The bottom right corner **Point**.
    pub fn bottom_right(&self) -> Point {
        [self.right(), self.bottom()]
    }

    /// The edges of the **Rect** in a tuple (top, bottom, left, right).
    pub fn l_r_b_t(&self) -> (Scalar, Scalar, Scalar, Scalar) {
        (self.left(), self.right(), self.bottom(), self.top())
    }

    /// The left and top edges of the **Rect** along with the width and height.
    pub fn l_t_w_h(&self) -> (Scalar, Scalar, Scalar, Scalar) {
        let (w, h) = self.w_h();
        (self.left(), self.top(), w, h)
    }

    /// The left and bottom edges of the **Rect** along with the width and height.
    pub fn l_b_w_h(&self) -> (Scalar, Scalar, Scalar, Scalar) {
        let (w, h) = self.w_h();
        (self.left(), self.bottom(), w, h)
    }

    /// Shift the Rect along the x axis.
    pub fn shift_x(self, x: Scalar) -> Self {
        Rect { x: self.x.shift(x), ..self }
    }

    /// Shift the Rect along the y axis.
    pub fn shift_y(self, y: Scalar) -> Self {
        Rect { y: self.y.shift(y), ..self }
    }

    /// Shift the Rect by the given Point.
    pub fn shift(self, xy: Point) -> Self {
        self.shift_x(xy[0]).shift_y(xy[1])
    }

    /// Returns a `Rect` with a position relative to the given position on the *x* axis.
    pub fn relative_to_x(self, x: Scalar) -> Self {
        Rect { x: self.x.shift(-x), ..self }
    }

    /// Returns a `Rect` with a position relative to the given position on the *y* axis.
    pub fn relative_to_y(self, y: Scalar) -> Self {
        Rect { y: self.y.shift(-y), ..self }
    }

    /// Returns a `Rect` with a position relative to the given position.
    pub fn relative_to(self, xy: Point) -> Self {
        self.relative_to_x(xy[0]).relative_to_y(xy[1])
    }

    /// Does the given point touch the Rectangle.
    pub fn is_over(&self, xy: Point) -> bool {
        self.x.is_over(xy[0]) && self.y.is_over(xy[1])
    }

    /// The Rect with some padding applied to the left edge.
    pub fn pad_left(self, pad: Scalar) -> Self {
        Rect { x: self.x.pad_start(pad), ..self }
    }

    /// The Rect with some padding applied to the right edge.
    pub fn pad_right(self, pad: Scalar) -> Self {
        Rect { x: self.x.pad_end(pad), ..self }
    }

    /// The rect with some padding applied to the bottom edge.
    pub fn pad_bottom(self, pad: Scalar) -> Self {
        Rect { y: self.y.pad_start(pad), ..self }
    }

    /// The Rect with some padding applied to the top edge.
    pub fn pad_top(self, pad: Scalar) -> Self {
        Rect { y: self.y.pad_end(pad), ..self }
    }

    /// The Rect with some padding amount applied to each edge.
    pub fn pad(self, pad: Scalar) -> Self {
        let Rect { x, y } = self;
        Rect { x: x.pad(pad), y: y.pad(pad) }
    }

    /// The Rect with some padding applied.
    pub fn padding(self, padding: Padding) -> Self {
        Rect {
            x: self.x.pad_ends(padding.x.start, padding.x.end),
            y: self.y.pad_ends(padding.y.start, padding.y.end),
        }
    }

    /// Stretches the closest edge(s) to the given point if the point lies outside of the Rect area.
    pub fn stretch_to_point(self, point: Point) -> Self {
        let Rect { x, y } = self;
        Rect {
            x: x.stretch_to_value(point[0]),
            y: y.stretch_to_value(point[1]),
        }
    }

    /// Align `self`'s right edge with the left edge of the `other` **Rect**.
    pub fn left_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_before(other.x),
            y: self.y,
        }
    }

    /// Align `self`'s left edge with the right dge of the `other` **Rect**.
    pub fn right_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_after(other.x),
            y: self.y,
        }
    }

    /// Align `self`'s top edge with the bottom edge of the `other` **Rect**.
    pub fn below(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_before(other.x),
        }
    }

    /// Align `self`'s bottom edge with the top edge of the `other` **Rect**.
    pub fn above(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_after(other.x),
        }
    }

    /// Align `self` to `other` along the *x* axis in accordance with the given `Align` variant.
    pub fn align_x_of(self, align: super::Align, other: Self) -> Self {
        Rect {
            x: self.x.align_to(align, other.x),
            y: self.y,
        }
    }

    /// Align `self` to `other` along the *y* axis in accordance with the given `Align` variant.
    pub fn align_y_of(self, align: super::Align, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_to(align, other.y),
        }
    }

    /// Align `self`'s left edge with the left edge of the `other` **Rect**.
    pub fn align_left_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_start_of(other.x),
            y: self.y,
        }
    }

    /// Align the middle of `self` with the middle of the `other` **Rect** along the *x* axis.
    pub fn align_middle_x_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_middle_of(other.x),
            y: self.y,
        }
    }

    /// Align `self`'s right edge with the right edge of the `other` **Rect**.
    pub fn align_right_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_end_of(other.x),
            y: self.y,
        }
    }

    /// Align `self`'s bottom edge with the bottom edge of the `other` **Rect**.
    pub fn align_bottom_of(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_start_of(other.y),
        }
    }

    /// Align the middle of `self` with the middle of the `other` **Rect** along the *y* axis.
    pub fn align_middle_y_of(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_middle_of(other.y),
        }
    }

    /// Align `self`'s top edge with the top edge of the `other` **Rect**.
    pub fn align_top_of(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_end_of(other.y),
        }
    }

    /// Place `self` along the top left edges of the `other` **Rect**.
    pub fn top_left_of(self, other: Self) -> Self {
        self.align_left_of(other).align_top_of(other)
    }

    /// Place `self` along the top right edges of the `other` **Rect**.
    pub fn top_right_of(self, other: Self) -> Self {
        self.align_right_of(other).align_top_of(other)
    }

    /// Place `self` along the bottom left edges of the `other` **Rect**.
    pub fn bottom_left_of(self, other: Self) -> Self {
        self.align_left_of(other).align_bottom_of(other)
    }

    /// Place `self` along the bottom right edges of the `other` **Rect**.
    pub fn bottom_right_of(self, other: Self) -> Self {
        self.align_right_of(other).align_bottom_of(other)
    }

    /// Place `self` in the middle of the top edge of the `other` **Rect**.
    pub fn mid_top_of(self, other: Self) -> Self {
        self.align_middle_x_of(other).align_top_of(other)
    }

    /// Place `self` in the middle of the bottom edge of the `other` **Rect**.
    pub fn mid_bottom_of(self, other: Self) -> Self {
        self.align_middle_x_of(other).align_bottom_of(other)
    }

    /// Place `self` in the middle of the left edge of the `other` **Rect**.
    pub fn mid_left_of(self, other: Self) -> Self {
        self.align_left_of(other).align_middle_y_of(other)
    }

    /// Place `self` in the middle of the right edge of the `other` **Rect**.
    pub fn mid_right_of(self, other: Self) -> Self {
        self.align_right_of(other).align_middle_y_of(other)
    }

    /// Place `self` directly in the middle of the `other` **Rect**.
    pub fn middle_of(self, other: Self) -> Self {
        self.align_middle_x_of(other).align_middle_y_of(other)
    }

    /// Return the **Corner** of `self` that is closest to the given **Point**.
    pub fn closest_corner(&self, xy: Point) -> Corner {
        use super::Edge;
        let x_edge = self.x.closest_edge(xy[0]);
        let y_edge = self.y.closest_edge(xy[1]);
        match (x_edge, y_edge) {
            (Edge::Start, Edge::Start) => Corner::BottomLeft,
            (Edge::Start, Edge::End) => Corner::TopLeft,
            (Edge::End, Edge::Start) => Corner::BottomRight,
            (Edge::End, Edge::End) => Corner::TopRight,
        }
    }

}
