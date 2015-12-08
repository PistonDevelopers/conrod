
use {CharacterCache, Ui};
use widget;

pub use self::range::{Edge, Range};
pub use self::rect::{Corner, Rect};
pub use self::rect::is_over as is_over_rect;
pub use self::matrix::Matrix;


pub mod matrix;
pub mod range;
pub mod rect;


/// An alias over the Scalar type used throughout Conrod.
///
/// This type is primarily used for spatial dimensions and positioning.
pub type Scalar = f64;

/// The depth at which the widget will be rendered.
///
/// This determines the order of rendering where widgets with a greater depth will be rendered
/// first.
///
/// 0.0 is the default depth.
pub type Depth = f32;

/// General use 2D spatial dimensions.
pub type Dimensions = [Scalar; 2];

/// General use 2D spatial point.
pub type Point = [Scalar; 2];

/// Some **Position** of some **Widget** along a single axis.
///
/// **Position**s for both the *x* and *y* axes are stored internally within the
/// **widget::CommonBuilder** type, allowing all widgets to be positioned in a variety of different
/// ways.
///
/// See the [**Positionable**](./trait.Positionable) trait for methods that allow for setting the
/// **Position**s in various ways.
///
/// Note that **Positionable** is implemented for *all* types that implement **Widget**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Position {
    /// A specific position.
    Absolute(Scalar),
    /// A position relative to some other Widget.
    Relative(Scalar, Option<widget::Index>),
    /// A position aligned with some other Widget.
    Align(Align, Option<widget::Index>),
    /// A direction relative to some other Widget.
    Direction(Direction, Scalar, Option<widget::Index>),
    /// A position at a place on some other Widget.
    Place(Place, Option<widget::Index>),
}

/// Directionally positioned, normally relative to some other widget.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Positioned forwards (*positive* **Scalar**) along some **Axis**.
    Forwards,
    /// Positioned backwards (*negative* **Scalar**) along some **Axis**.
    Backwards,
}

/// The orientation of **Align**ment along some **Axis**.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Align {
    /// **Align** our **Start** with the **Start** of some other widget along the **Axis**.
    Start,
    /// **Align** our **Middle** with the **Middle** of some other widget along the **Axis**.
    Middle,
    /// **Align** our **End** with the **End** of some other widget along the **Axis**.
    End,
}

/// Place the widget at a position on some other widget.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Place {
    /// Place upon the **Start** of the Widget's `kid_area`.
    Start,
    /// Place upon the **Middle** of the Widget's `kid_area`.
    Middle,
    /// Place upon the **End** of the Widget's `kid_area`.
    End,
}

/// The length of a **Widget** over either the *x* or *y* axes.
///
/// This type is used to represent the different ways in which a dimension may be sized.
///
/// See the [**Sizeable**](./trait.Sizeable) trait for methods that allow for setting the
/// `x` and `y` **Dimension**s in various ways.
///
/// Note that **Sizeable** is implemented for *all* types that implement **Widget**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Dimension {
    /// Some specific length has been given.
    Absolute(Scalar),
    /// The dimension should match that of the widget at the given index.
    Of(widget::Index),
    /// The dimension should match that of the `kid_area` of the widget at the given index.
    KidAreaOf(widget::Index),
}

/// Widgets that are positionable.
///
/// A **Position** is stored internally within the **widget::CommonBuilder** type, allowing all
/// widgets to be positioned in a variety of different ways.
///
/// Thus, **Positionable** can be implemented for *all* types that implement **Widget**.
pub trait Positionable: Sized {

    /// Build with the given **Position** along the *x* axis.
    fn x_position(self, Position) -> Self;

    /// Build with the given **Position** along the *y* axis.
    fn y_position(self, Position) -> Self;

    /// Get the **Position** along the *x* axis.
    fn get_x_position<C: CharacterCache>(&self, ui: &Ui<C>) -> Position;

    /// Get the **Position** along the *y* axis.
    fn get_y_position<C: CharacterCache>(&self, ui: &Ui<C>) -> Position;

    // Absolute positioning.

    /// Build with the given **Absolute** **Position** along the *x* axis.
    fn x(self, x: Scalar) -> Self {
        self.x_position(Position::Absolute(x))
    }

    /// Build with the given **Absolute** **Position** along the *y* axis.
    fn y(self, y: Scalar) -> Self {
        self.y_position(Position::Absolute(y))
    }

    /// Set the **Position** with some Point.
    fn xy(self, point: Point) -> Self {
        self.x(point[0]).y(point[1])
    }

    /// Set the **Position** with *x* *y* coordinates.
    fn x_y(self, x: Scalar, y: Scalar) -> Self {
        self.xy([x, y])
    }

    // Relative positioning.

    /// Set the **Position** along the *x* axis **Relative** to the previous widget.
    fn x_relative(self, x: Scalar) -> Self {
        self.x_position(Position::Relative(x, None))
    }

    /// Set the **Position** along the *y* axis **Relative** to the previous widget.
    fn y_relative(self, y: Scalar) -> Self {
        self.y_position(Position::Relative(y, None))
    }

    /// Set the **Position** **Relative** to the previous widget.
    fn xy_relative(self, point: Point) -> Self {
        self.x_relative(point[0]).y_relative(point[1])
    }

    /// Set the **Position** **Relative** to the previous widget.
    fn x_y_relative(self, x: Scalar, y: Scalar) -> Self {
        self.xy_relative([x, y])
    }

    /// Set the position relative to the widget with the given widget::Index.
    fn x_relative_to<I: Into<widget::Index>>(self, other: I, x: Scalar) -> Self {
        self.x_position(Position::Relative(x, Some(other.into())))
    }

    /// Set the position relative to the widget with the given widget::Index.
    fn y_relative_to<I: Into<widget::Index>>(self, other: I, y: Scalar) -> Self {
        self.y_position(Position::Relative(y, Some(other.into())))
    }

    /// Set the position relative to the widget with the given widget::Index.
    fn xy_relative_to<I: Into<widget::Index> + Copy>(self, other: I, xy: Point) -> Self {
        self.x_relative_to(other, xy[0]).y_relative_to(other, xy[1])
    }

    /// Set the position relative to the widget with the given widget::Index.
    fn x_y_relative_to<I: Into<widget::Index> + Copy>(self, other: I, x: Scalar, y: Scalar) -> Self {
        self.xy_relative_to(other, [x, y])
    }

    // Directional positioning.

    /// Build with the **Position** along the *x* axis as some distance from another widget.
    fn x_direction(self, direction: Direction, x: Scalar) -> Self {
        self.x_position(Position::Direction(direction, x, None)).align_top()
    }

    /// Build with the **Position** along the *y* axis as some distance from another widget.
    fn y_direction(self, direction: Direction, y: Scalar) -> Self {
        self.y_position(Position::Direction(direction, y, None)).align_left()
    }

    /// Build with the **Position** as some distance below another widget.
    fn down(self, y: Scalar) -> Self {
        self.y_direction(Direction::Backwards, y)
    }

    /// Build with the **Position** as some distance above another widget.
    fn up(self, y: Scalar) -> Self {
        self.y_direction(Direction::Forwards, y)
    }

    /// Build with the **Position** as some distance to the left of another widget.
    fn left(self, x: Scalar) -> Self {
        self.x_direction(Direction::Backwards, x)
    }

    /// Build with the **Position** as some distance to the right of another widget.
    fn right(self, x: Scalar) -> Self {
        self.x_direction(Direction::Forwards, x)
    }

    /// Build with the **Position** along the *x* axis as some distance from the given widget.
    fn x_direction_from<I>(self, other: I, direction: Direction, x: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.x_position(Position::Direction(direction, x, Some(other.into()))).align_top_of(other)
    }

    /// Build with the **Position** along the *y* axis as some distance from the given widget.
    fn y_direction_from<I>(self, other: I, direction: Direction, y: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.y_position(Position::Direction(direction, y, Some(other.into()))).align_left_of(other)
    }

    /// Build with the **Position** as some distance below the given widget.
    fn down_from<I: Into<widget::Index> + Copy>(self, other: I, y: Scalar) -> Self {
        self.y_direction_from(other, Direction::Backwards, y)
    }

    /// Build with the **Position** as some distance above the given widget.
    fn up_from<I: Into<widget::Index> + Copy>(self, other: I, y: Scalar) -> Self {
        self.y_direction_from(other, Direction::Forwards, y)
    }

    /// Build with the **Position** as some distance to the left of the given widget.
    fn left_from<I: Into<widget::Index> + Copy>(self, other: I, x: Scalar) -> Self {
        self.x_direction_from(other, Direction::Backwards, x)
    }

    /// Build with the **Position** as some distance to the right of the given widget.
    fn right_from<I: Into<widget::Index> + Copy>(self, other: I, x: Scalar) -> Self {
        self.x_direction_from(other, Direction::Forwards, x)
    }

    // Alignment positioning.

    /// Align the **Position** of the widget along the *x* axis.
    fn x_align(self, align: Align) -> Self {
        self.x_position(Position::Align(align, None))
    }

    /// Align the **Position** of the widget along the *y* axis.
    fn y_align(self, align: Align) -> Self {
        self.y_position(Position::Align(align, None))
    }

    /// Align the position to the left (only effective for Up or Down `Direction`s).
    fn align_left(self) -> Self {
        self.x_align(Align::Start)
    }

    /// Align the position to the middle (only effective for Up or Down `Direction`s).
    fn align_middle_x(self) -> Self {
        self.x_align(Align::Middle)
    }

    /// Align the position to the right (only effective for Up or Down `Direction`s).
    fn align_right(self) -> Self {
        self.x_align(Align::End)
    }

    /// Align the position to the top (only effective for Left or Right `Direction`s).
    fn align_top(self) -> Self {
        self.y_align(Align::End)
    }

    /// Align the position to the middle (only effective for Left or Right `Direction`s).
    fn align_middle_y(self) -> Self {
        self.y_align(Align::Middle)
    }

    /// Align the position to the bottom (only effective for Left or Right `Direction`s).
    fn align_bottom(self) -> Self {
        self.y_align(Align::Start)
    }

    /// Align the **Position** of the widget with the given widget along the *x* axis.
    fn x_align_to<I: Into<widget::Index>>(self, other: I, align: Align) -> Self {
        self.x_position(Position::Align(align, Some(other.into())))
    }

    /// Align the **Position** of the widget with the given widget along the *y* axis.
    fn y_align_to<I: Into<widget::Index>>(self, other: I, align: Align) -> Self {
        self.y_position(Position::Align(align, Some(other.into())))
    }

    /// Align the position to the left (only effective for Up or Down `Direction`s).
    fn align_left_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.x_align_to(other, Align::Start)
    }

    /// Align the position to the middle (only effective for Up or Down `Direction`s).
    fn align_middle_x_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.x_align_to(other, Align::Middle)
    }

    /// Align the position to the right (only effective for Up or Down `Direction`s).
    fn align_right_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.x_align_to(other, Align::End)
    }

    /// Align the position to the top (only effective for Left or Right `Direction`s).
    fn align_top_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.y_align_to(other, Align::End)
    }

    /// Align the position to the middle (only effective for Left or Right `Direction`s).
    fn align_middle_y_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.y_align_to(other, Align::Middle)
    }

    /// Align the position to the bottom (only effective for Left or Right `Direction`s).
    fn align_bottom_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.y_align_to(other, Align::Start)
    }

    ///// `Place` methods. /////

    /// Place the widget at some position on the `other` Widget along the *x* axis.
    fn x_place_on<I: Into<widget::Index>>(self, other: I, place: Place) -> Self {
        self.x_position(Position::Place(place, Some(other.into())))
    }

    /// Place the widget at some position on the `other` Widget along the *y* axis.
    fn y_place_on<I: Into<widget::Index>>(self, other: I, place: Place) -> Self {
        self.y_position(Position::Place(place, Some(other.into())))
    }

    /// Place the widget in the middle of the given Widget.
    fn middle_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::Middle).y_place_on(other, Place::Middle)
    }

    /// Place the widget in the top left corner of the given Widget.
    fn top_left_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::Start).y_place_on(other, Place::End)
    }

    /// Place the widget in the top right corner of the given Widget.
    fn top_right_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::End).y_place_on(other, Place::End)
    }

    /// Place the widget in the bottom left corner of the given Widget.
    fn bottom_left_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::Start).y_place_on(other, Place::Start)
    }

    /// Place the widget in the bottom right corner of the given Widget.
    fn bottom_right_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::End).y_place_on(other, Place::Start)
    }

    /// Place the widget in the middle of the top edge of the given Widget.
    fn mid_top_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::Middle).y_place_on(other, Place::End)
    }

    /// Place the widget in the middle of the bottom edge of the given Widget.
    fn mid_bottom_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::Middle).y_place_on(other, Place::Start)
    }

    /// Place the widget in the middle of the left edge of the given Widget.
    fn mid_left_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::Start).y_place_on(other, Place::Middle)
    }

    /// Place the widget in the middle of the right edge of the given Widget.
    fn mid_right_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::End).y_place_on(other, Place::Middle)
    }

    /// Place the widget at some position on the Widget along the *x* axis.
    fn x_place(self, place: Place) -> Self {
        self.x_position(Position::Place(place, None))
    }

    /// Place the widget at some position on the Widget along the *y* axis.
    fn y_place(self, place: Place) -> Self {
        self.y_position(Position::Place(place, None))
    }

    /// Place the widget in the middle of the current parent Widget.
    fn middle(self) -> Self {
        self.x_place(Place::Middle).y_place(Place::Middle)
    }

    /// Place the widget in the top left corner of the current parent Widget.
    fn top_left(self) -> Self {
        self.x_place(Place::Start).y_place(Place::End)
    }

    /// Place the widget in the top right corner of the current parent Widget.
    fn top_right(self) -> Self {
        self.x_place(Place::End).y_place(Place::End)
    }

    /// Place the widget in the bottom left corner of the current parent Widget.
    fn bottom_left(self) -> Self {
        self.x_place(Place::Start).y_place(Place::Start)
    }

    /// Place the widget in the bottom right corner of the current parent Widget.
    fn bottom_right(self) -> Self {
        self.x_place(Place::End).y_place(Place::Start)
    }

    /// Place the widget in the middle of the top edge of the current parent Widget.
    fn mid_top(self) -> Self {
        self.x_place(Place::Middle).y_place(Place::End)
    }

    /// Place the widget in the middle of the bottom edge of the current parent Widget.
    fn mid_bottom(self) -> Self {
        self.x_place(Place::Middle).y_place(Place::Start)
    }

    /// Place the widget in the middle of the left edge of the current parent Widget.
    fn mid_left(self) -> Self {
        self.x_place(Place::Start).y_place(Place::Middle)
    }

    /// Place the widget in the middle of the right edge of the current parent Widget.
    fn mid_right(self) -> Self {
        self.x_place(Place::End).y_place(Place::Middle)
    }

    ///// Rendering Depth (aka Z axis) /////

    /// The depth at which the widget should be rendered relatively to its sibling widgets.
    fn depth(self, depth: Depth) -> Self;

    /// Return the depth.
    fn get_depth(&self) -> Depth;

}

/// Widgets that support different dimensions.
pub trait Sizeable: Sized {

    // Required implementations.

    /// Set the length along the x axis.
    fn x_dimension(self, x: Dimension) -> Self;

    /// Set the length along the y axis.
    fn y_dimension(self, x: Dimension) -> Self;

    /// The widget's length along the x axis as a Dimension.
    fn get_x_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension;

    /// The widget's length along the y axis as a Dimension.
    fn get_y_dimension<C: CharacterCache>(&self, ui: &Ui<C>) -> Dimension;

    // Provided defaults.

    /// Set the absolute width for the widget.
    #[inline]
    fn width(self, w: Scalar) -> Self {
        self.x_dimension(Dimension::Absolute(w))
    }

    /// Set the absolute height for the widget.
    #[inline]
    fn height(self, h: Scalar) -> Self {
        self.y_dimension(Dimension::Absolute(h))
    }

    /// Set the dimensions for the widget.
    #[inline]
    fn dim(self, dim: Dimensions) -> Self {
        self.width(dim[0]).height(dim[1])
    }

    /// Set the width and height for the widget.
    #[inline]
    fn dimensions(self, width: Scalar, height: Scalar) -> Self {
        self.dim([width, height])
    }

    /// Set the width as the width of the widget at the given index.
    #[inline]
    fn width_of<I: Into<widget::Index>>(self, idx: I) -> Self {
        self.x_dimension(Dimension::Of(idx.into()))
    }

    /// Set the height as the height of the widget at the given index.
    #[inline]
    fn height_of<I: Into<widget::Index>>(self, idx: I) -> Self {
        self.y_dimension(Dimension::Of(idx.into()))
    }

    /// Set the dimensions as the dimensions of the widget at the given index.
    #[inline]
    fn dim_of<I: Into<widget::Index> + Copy>(self, idx: I) -> Self {
        self.width_of(idx).height_of(idx)
    }

    /// Set the width as the width of the padded area of the widget at the given index.
    #[inline]
    fn kid_area_width_of<I: Into<widget::Index>>(self, idx: I) -> Self {
        self.x_dimension(Dimension::KidAreaOf(idx.into()))
    }

    /// Set the height as the height of the padded area of the widget at the given index.
    #[inline]
    fn kid_area_height_of<I: Into<widget::Index>>(self, idx: I) -> Self {
        self.y_dimension(Dimension::KidAreaOf(idx.into()))
    }

    /// Set the dimensions as the dimensions of the padded area of the widget at the given index.
    #[inline]
    fn kid_area_dim_of<I: Into<widget::Index> + Copy>(self, idx: I) -> Self {
        self.kid_area_width_of(idx).kid_area_height_of(idx)
    }

    /// Get the absolute width of the widget as a Scalar value.
    #[inline]
    fn get_width<C: CharacterCache>(&self, ui: &Ui<C>) -> Option<Scalar> {
        match self.get_x_dimension(ui) {
            Dimension::Absolute(width) => Some(width),
            Dimension::Of(idx) => ui.width_of(idx),
            Dimension::KidAreaOf(idx) => ui.kid_area_of(idx).map(|r| r.w()),
        }
    }

    /// Get the height of the widget.
    #[inline]
    fn get_height<C: CharacterCache>(&self, ui: &Ui<C>) -> Option<Scalar> {
        match self.get_y_dimension(ui) {
            Dimension::Absolute(height) => Some(height),
            Dimension::Of(idx) => ui.height_of(idx),
            Dimension::KidAreaOf(idx) => ui.kid_area_of(idx).map(|r| r.h()),
        }
    }

    /// The dimensions for the widget.
    #[inline]
    fn get_dim<C: CharacterCache>(&self, ui: &Ui<C>) -> Option<Dimensions> {
        self.get_width(ui).and_then(|w| self.get_height(ui).map(|h| [w, h]))
    }

}

/// Return which corner of the rectangle the given Point is within.
pub fn corner(xy: Point, dim: Dimensions) -> Corner {
    Rect::from_xy_dim([0.0, 0.0], dim).closest_corner(xy)
}


/// The x offset required to align an element with `width` to the left of a target element.
pub fn align_left_of(target_width: Scalar, width: Scalar) -> Scalar {
    width / 2.0 - target_width / 2.0
}

/// The x offset required to align an element with `width` to the right of a target element.
pub fn align_right_of(target_width: Scalar, width: Scalar) -> Scalar {
    target_width / 2.0 - width / 2.0
}

/// The y offset required to align an element with `height` to the bottom of a target element.
pub fn align_bottom_of(target_height: Scalar, height: Scalar) -> Scalar {
    height / 2.0 - target_height / 2.0
}

/// The y offset required to align an element with `height` to the top of a target element.
pub fn align_top_of(target_height: Scalar, height: Scalar) -> Scalar {
    target_height / 2.0 - height / 2.0
}


/// The position of a rect with `dim` Dimensions at the middle of the `target` Dimensions.
pub fn middle_of(_target: Dimensions, _dim: Dimensions) -> Point {
    [0.0, 0.0]
}

/// The position of a rect with `dim` Dimensions at the top left of the `target` Dimensions.
pub fn top_left_of(target: Dimensions, dim: Dimensions) -> Point {
    [align_left_of(target[0], dim[0]), align_top_of(target[1], dim[1])]
}

/// The position of a rect with `dim` Dimensions at the top right of the `target` Dimensions.
pub fn top_right_of(target: Dimensions, dim: Dimensions) -> Point {
    [align_right_of(target[0], dim[0]), align_top_of(target[1], dim[1])]
}

/// The position of a rect with `dim` Dimensions at the bottom left of the `target` Dimensions.
pub fn bottom_left_of(target: Dimensions, dim: Dimensions) -> Point {
    [align_left_of(target[0], dim[0]), align_bottom_of(target[1], dim[1])]
}

/// The position of a rect with `dim` Dimensions at the bottom right of the `target` Dimensions.
pub fn bottom_right_of(target: Dimensions, dim: Dimensions) -> Point {
    [align_right_of(target[0], dim[0]), align_bottom_of(target[1], dim[1])]
}

/// The position of a rect with `dim` Dimensions at the middle of the inside of the top edge of
/// the `target` Dimensions.
pub fn mid_top_of(target: Dimensions, dim: Dimensions) -> Point {
    [0.0, align_top_of(target[1], dim[1])]
}

/// The position of a rect with `dim` Dimensions at the middle of the inside of the bottom edge of
/// the `target` Dimensions.
pub fn mid_bottom_of(target: Dimensions, dim: Dimensions) -> Point {
    [0.0, align_bottom_of(target[1], dim[1])]
}

/// The position of a rect with `dim` Dimensions at the middle of the inside of the left edge of
/// the `target` Dimensions.
pub fn mid_left_of(target: Dimensions, dim: Dimensions) -> Point {
    [align_left_of(target[0], dim[0]), 0.0]
}

/// The position of a rect with `dim` Dimensions at the middle of the inside of the right edge of
/// the `target` Dimensions.
pub fn mid_right_of(target: Dimensions, dim: Dimensions) -> Point {
    [align_right_of(target[0], dim[0]), 0.0]
}


/// The distance between the inner edge of a frame and the outer edge of the inner content.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Padding {
    /// Padding between the top of a Widget and the top of a parent Widget.
    pub top: f64,
    /// Padding between the bottom of a Widget and the bottom of a parent Widget.
    pub bottom: f64,
    /// Margin between the left of a Widget and the left of a parent Widget.
    pub left: f64,
    /// Margin between the right of a Widget and the right of a parent Widget.
    pub right: f64,
}

impl Padding {

    /// No padding.
    pub fn none() -> Padding {
        Padding { top: 0.0, bottom: 0.0, left: 0.0, right: 0.0 }
    }

}

/// The distance between the dimension bound and the outer edge of the frame.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Margin {
    /// Margin between the y max parent Widget and the outer edge of its frame.
    pub top: f64,
    /// Margin between the y min parent Widget and the outer edge of its frame.
    pub bottom: f64,
    /// Margin between the x min parent Widget and the outer edge of its frame.
    pub left: f64,
    /// Margin between the x max parent Widget and the outer edge of its frame.
    pub right: f64,
}


impl Margin {
    /// No margin.
    pub fn none() -> Margin {
        Margin { top: 0.0, bottom: 0.0, left: 0.0, right: 0.0 }
    }
}

