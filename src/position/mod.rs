
use {CharacterCache, Theme, Ui};
use widget;

pub use self::range::Range;
pub use self::rect::Rect;
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

/// The **Position** argument used to represent the positioning of a **Widget**.
///
/// A **Position** is stored internally within the **widget::CommonBuilder** type, allowing all
/// widgets to be positioned in a variety of different ways.
///
/// See the [**Positionable**](./trait.Positionable) trait for methods that allow for setting the
/// **Position** in various ways.
///
/// Note that **Positionable** is implemented for *all* types that implement **Widget**.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Position {
    /// A specific position.
    Absolute(Scalar, Scalar),
    /// A position relative to some other Widget.
    Relative(Scalar, Scalar, Option<widget::Index>),
    /// A direction relative to some other Widget.
    Direction(Direction, Scalar, Option<widget::Index>),
    /// A position at a place on some other Widget.
    Place(Place, Option<widget::Index>),
}

/// The length of a **Widget** over either the *x* or *y* axes.
///
/// This type is used to represent the different ways in which a dimension may be sized.
///
/// See the [**Sizeable**](./trait.Sizeable) trait for methods that allow for setting the
/// `x` and `y` **Dimension**s in various ways.
///
/// Note that **Sizeable** is implemented for *all* types that implement **Widget**.
#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Dimension {
    /// Some specific length has been given.
    Absolute(Scalar),
    /// The dimension should match that of the widget at the given index.
    Of(widget::Index),
    /// The dimension should match that of the `kid_area` of the widget at the given index.
    KidAreaOf(widget::Index),
}


impl Position {
    /// The default widget Position.
    pub fn default() -> Position{
        Position::Direction(Direction::Down, 20.0, None)
    }
}


/// Directionally positioned, relative to another widget.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable, PartialEq, Eq)]
pub enum Direction {
    /// Positioned above.
    Up,
    /// Positioned below.
    Down,
    /// Positioned to the left.
    Left,
    /// Positioned to the right.
    Right,
}

/// The horizontal alignment of a widget positioned relatively to another UI element on the y axis.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable, PartialEq, Eq)]
pub struct HorizontalAlign(pub Horizontal, pub Option<widget::Index>);

/// The orientation of a HorizontalAlign.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable, PartialEq, Eq)]
pub enum Horizontal {
    /// Align the left edges of the widgets.
    Left,
    /// Align the centres of the widgets' closest parallel edges.
    Middle,
    /// Align the right edges of the relative widgets.
    Right,
}

/// The vertical alignment of a widget positioned relatively to another UI element on the x axis.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable, PartialEq, Eq)]
pub struct VerticalAlign(pub Vertical, pub Option<widget::Index>);

/// The orientation of a VerticalAlign.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable, PartialEq, Eq)]
pub enum Vertical {
    /// Align the top edges of the widgets.
    Top,
    /// Align the centres of the widgets' closest parallel edges.
    Middle,
    /// Align the bottom edges of the widgets.
    Bottom,
}

/// Place the widget at a position on some other widget.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable, PartialEq, Eq)]
pub enum Place {
    /// Centre of the Widget.
    Middle,
    /// Top left of the Widget - pad_top + pad_left.
    TopLeft,
    /// Top right of the Widget - pad_top - pad_right.
    TopRight,
    /// Bottom left of the Widget + pad_bottom + pad_left.
    BottomLeft,
    /// Bottom right of the Widget + pad_bottom - pad_right.
    BottomRight,
    /// Top centre of the Widget - pad_top.
    MidTop,
    /// Bottom centre of the Widget + pad_bottom.
    MidBottom,
    /// Left centre of the Widget + pad_left.
    MidLeft,
    /// Right centre of the Widget - pad_right.
    MidRight,
}

/// Widgets that are positionable.
///
/// A **Position** is stored internally within the **widget::CommonBuilder** type, allowing all
/// widgets to be positioned in a variety of different ways.
///
/// Thus, **Positionable** can be implemented for *all* types that implement **Widget**.
pub trait Positionable: Sized {

    /// Set the Position.
    fn position(self, pos: Position) -> Self;

    /// Get the Position.
    fn get_position(&self, theme: &Theme) -> Position;

    /// Set the position with some Point.
    fn point(self, point: Point) -> Self {
        self.position(Position::Absolute(point[0], point[1]))
    }

    /// Set the position with XY co-ords.
    fn xy(self, x: Scalar, y: Scalar) -> Self {
        self.position(Position::Absolute(x, y))
    }

    /// Set the point relative to the previous widget.
    fn relative(self, point: Point) -> Self {
        self.position(Position::Relative(point[0], point[1], None))
    }

    /// Set the xy relative to the previous widget.
    fn relative_xy(self, x: Scalar, y: Scalar) -> Self {
        self.position(Position::Relative(x, y, None))
    }

    /// Set the position relative to the widget with the given widget::Index.
    fn relative_to<I: Into<widget::Index>>(self, other: I, point: Point) -> Self {
        self.position(Position::Relative(point[0], point[1], Some(other.into())))
    }

    /// Set the position relative to the widget with the given widget::Index.
    fn relative_xy_to<I: Into<widget::Index>>(self, other: I, x: Scalar, y: Scalar) -> Self {
        self.position(Position::Relative(x, y, Some(other.into())))
    }

    /// Set the position as below the previous widget.
    fn down(self, pixels: Scalar) -> Self {
        self.position(Position::Direction(Direction::Down, pixels, None))
    }

    /// Set the position as above the previous widget.
    fn up(self, pixels: Scalar) -> Self {
        self.position(Position::Direction(Direction::Up, pixels, None))
    }

    /// Set the position to the left of the previous widget.
    fn left(self, pixels: Scalar) -> Self {
        self.position(Position::Direction(Direction::Left, pixels, None))
    }

    /// Set the position to the right of the previous widget.
    fn right(self, pixels: Scalar) -> Self {
        self.position(Position::Direction(Direction::Right, pixels, None))
    }

    /// Set the position as below the widget with the given widget::Index.
    fn down_from<I: Into<widget::Index>>(self, other: I, pixels: Scalar) -> Self {
        self.position(Position::Direction(Direction::Down, pixels, Some(other.into())))
    }

    /// Set the position as above the widget with the given widget::Index.
    fn up_from<I: Into<widget::Index>>(self, other: I, pixels: Scalar) -> Self {
        self.position(Position::Direction(Direction::Up, pixels, Some(other.into())))
    }

    /// Set the position to the left of the widget with the given widget::Index.
    fn left_from<I: Into<widget::Index>>(self, other: I, pixels: Scalar) -> Self {
        self.position(Position::Direction(Direction::Left, pixels, Some(other.into())))
    }

    /// Set the position to the right of the widget with the given widget::Index.
    fn right_from<I: Into<widget::Index>>(self, other: I, pixels: Scalar) -> Self {
        self.position(Position::Direction(Direction::Right, pixels, Some(other.into())))
    }

    ///// `Align` methods. /////

    /// Align the position horizontally (only effective for Up or Down `Direction`s).
    fn horizontal_align(self, align: HorizontalAlign) -> Self;

    /// Align the position vertically (only effective for Left or Right `Direction`s).
    fn vertical_align(self, align: VerticalAlign) -> Self;

    /// Return the horizontal alignment.
    fn get_horizontal_align(&self, theme: &Theme) -> HorizontalAlign;

    /// Return the vertical alignment.
    fn get_vertical_align(&self, theme: &Theme) -> VerticalAlign;

    /// Return the alignment of both axis.
    fn get_alignment(&self, theme: &Theme) -> (HorizontalAlign, VerticalAlign) {
        (self.get_horizontal_align(theme), self.get_vertical_align(theme))
    }

    /// Align the position to the left (only effective for Up or Down `Direction`s).
    fn align_left(self) -> Self {
        self.horizontal_align(HorizontalAlign(Horizontal::Left, None))
    }

    /// Align the position to the middle (only effective for Up or Down `Direction`s).
    fn align_middle_x(self) -> Self {
        self.horizontal_align(HorizontalAlign(Horizontal::Middle, None))
    }

    /// Align the position to the right (only effective for Up or Down `Direction`s).
    fn align_right(self) -> Self {
        self.horizontal_align(HorizontalAlign(Horizontal::Right, None))
    }

    /// Align the position to the top (only effective for Left or Right `Direction`s).
    fn align_top(self) -> Self {
        self.vertical_align(VerticalAlign(Vertical::Top, None))
    }

    /// Align the position to the middle (only effective for Left or Right `Direction`s).
    fn align_middle_y(self) -> Self {
        self.vertical_align(VerticalAlign(Vertical::Middle, None))
    }

    /// Align the position to the bottom (only effective for Left or Right `Direction`s).
    fn align_bottom(self) -> Self {
        self.vertical_align(VerticalAlign(Vertical::Bottom, None))
    }

    /// Align the position to the left (only effective for Up or Down `Direction`s).
    fn align_left_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.horizontal_align(HorizontalAlign(Horizontal::Left, Some(other.into())))
    }

    /// Align the position to the middle (only effective for Up or Down `Direction`s).
    fn align_middle_x_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.horizontal_align(HorizontalAlign(Horizontal::Middle, Some(other.into())))
    }

    /// Align the position to the right (only effective for Up or Down `Direction`s).
    fn align_right_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.horizontal_align(HorizontalAlign(Horizontal::Right, Some(other.into())))
    }

    /// Align the position to the top (only effective for Left or Right `Direction`s).
    fn align_top_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.vertical_align(VerticalAlign(Vertical::Top, Some(other.into())))
    }

    /// Align the position to the middle (only effective for Left or Right `Direction`s).
    fn align_middle_y_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.vertical_align(VerticalAlign(Vertical::Middle, Some(other.into())))
    }

    /// Align the position to the bottom (only effective for Left or Right `Direction`s).
    fn align_bottom_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.vertical_align(VerticalAlign(Vertical::Bottom, Some(other.into())))
    }

    ///// `Place` methods. /////

    /// Place the widget at some position on the Widget.
    fn place(self, place: Place, maybe_idx: Option<widget::Index>) -> Self {
        self.position(Position::Place(place, maybe_idx))
    }

    /// Place the widget in the middle of the given Widget.
    fn middle_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.place(Place::Middle, Some(other.into()))
    }

    /// Place the widget in the top left corner of the given Widget.
    fn top_left_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.place(Place::TopLeft, Some(other.into()))
    }

    /// Place the widget in the top right corner of the given Widget.
    fn top_right_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.place(Place::TopRight, Some(other.into()))
    }

    /// Place the widget in the bottom left corner of the given Widget.
    fn bottom_left_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.place(Place::BottomLeft, Some(other.into()))
    }

    /// Place the widget in the bottom right corner of the given Widget.
    fn bottom_right_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.place(Place::BottomRight, Some(other.into()))
    }

    /// Place the widget in the middle of the top edge of the given Widget.
    fn mid_top_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.place(Place::MidTop, Some(other.into()))
    }

    /// Place the widget in the middle of the bottom edge of the given Widget.
    fn mid_bottom_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.place(Place::MidBottom, Some(other.into()))
    }

    /// Place the widget in the middle of the left edge of the given Widget.
    fn mid_left_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.place(Place::MidLeft, Some(other.into()))
    }

    /// Place the widget in the middle of the right edge of the given Widget.
    fn mid_right_of<I: Into<widget::Index>>(self, other: I) -> Self {
        self.place(Place::MidRight, Some(other.into()))
    }

    /// Place the widget in the middle of the current parent Widget.
    fn middle(self) -> Self { self.place(Place::Middle, None) }

    /// Place the widget in the top left corner of the current parent Widget.
    fn top_left(self) -> Self { self.place(Place::TopLeft, None) }

    /// Place the widget in the top right corner of the current parent Widget.
    fn top_right(self) -> Self { self.place(Place::TopRight, None) }

    /// Place the widget in the bottom left corner of the current parent Widget.
    fn bottom_left(self) -> Self { self.place(Place::BottomLeft, None) }

    /// Place the widget in the bottom right corner of the current parent Widget.
    fn bottom_right(self) -> Self { self.place(Place::BottomRight, None) }

    /// Place the widget in the middle of the top edge of the current parent Widget.
    fn mid_top(self) -> Self { self.place(Place::MidTop, None) }

    /// Place the widget in the middle of the bottom edge of the current parent Widget.
    fn mid_bottom(self) -> Self { self.place(Place::MidBottom, None) }

    /// Place the widget in the middle of the left edge of the current parent Widget.
    fn mid_left(self) -> Self { self.place(Place::MidLeft, None) }

    /// Place the widget in the middle of the right edge of the current parent Widget.
    fn mid_right(self) -> Self { self.place(Place::MidRight, None) }

    ///// Rendering Depth (aka Z axis) /////

    /// The depth at which the widget should be rendered.
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

/// A corner of a rectangle.
#[derive(Copy, Clone)]
pub enum Corner {
    /// The top left quarter of a rectangle's area.
    TopLeft,
    /// The top right quarter of a rectangle's area.
    TopRight,
    /// The bottom left quarter of a rectangle's area.
    BottomLeft,
    /// The bottom right quarter of a rectangle's area.
    BottomRight,
}

/// Return which corner of the rectangle the given Point is within.
pub fn corner(xy: Point, dim: Dimensions) -> Corner {
    use utils::map_range;
    let x_perc = map_range(xy[0], -dim[0] / 2.0, dim[0] / 2.0, -1.0, 1.0);
    let y_perc = map_range(xy[1], -dim[1] / 2.0, dim[1] / 2.0, -1.0, 1.0);
    if      x_perc <= 0.0 && y_perc <= 0.0 { Corner::BottomLeft }
    else if x_perc >  0.0 && y_perc <= 0.0 { Corner::BottomRight }
    else if x_perc <= 0.0 && y_perc >  0.0 { Corner::TopLeft }
    else                                   { Corner::TopRight }
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

impl Horizontal {
    /// Align `width` to the given `target_width`.
    pub fn to(&self, target_width: Scalar, width: Scalar) -> Scalar {
        match *self {
            Horizontal::Left => align_left_of(target_width, width),
            Horizontal::Right => align_right_of(target_width, width),
            Horizontal::Middle => 0.0,
        }
    }
}

impl Vertical {
    /// Align `height` to the given `target_height`.
    pub fn to(&self, target_height: Scalar, height: Scalar) -> Scalar {
        match *self {
            Vertical::Top => align_top_of(target_height, height),
            Vertical::Bottom => align_bottom_of(target_height, height),
            Vertical::Middle => 0.0,
        }
    }
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

impl Place {
    /// Place the given `dim` within the `target_dim`.
    pub fn within(&self, target_dim: Dimensions, dim: Dimensions) -> Point {
        match *self {
            Place::Middle      => middle_of(target_dim, dim),
            Place::TopLeft     => top_left_of(target_dim, dim),
            Place::TopRight    => top_right_of(target_dim, dim),
            Place::BottomLeft  => bottom_left_of(target_dim, dim),
            Place::BottomRight => bottom_right_of(target_dim, dim),
            Place::MidTop      => mid_top_of(target_dim, dim),
            Place::MidBottom   => mid_bottom_of(target_dim, dim),
            Place::MidLeft     => mid_left_of(target_dim, dim),
            Place::MidRight    => mid_right_of(target_dim, dim),
        }
    }
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

    /// Determine the offset for the given `Place`.
    pub fn offset_from(&self, place: Place) -> Point {
        match place {
            Place::Middle => [0.0, 0.0],
            Place::TopLeft => [self.left, -self.top],
            Place::TopRight => [-self.right, -self.top],
            Place::BottomLeft => [self.left, self.bottom],
            Place::BottomRight => [-self.right, self.bottom],
            Place::MidTop => [0.0, -self.top],
            Place::MidBottom => [0.0, self.bottom],
            Place::MidLeft => [self.left, 0.0],
            Place::MidRight => [-self.right, 0.0],
        }
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
