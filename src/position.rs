
use graphics::character::CharacterCache;
use graphics::math::Scalar;
use theme::Theme;
use ui::GlyphCache;
use widget;

/// The depth at which the widget will be rendered. This determines the order of rendering where
/// widgets with a greater depth will be rendered first. 0.0 is the default depth.
pub type Depth = f32;

/// General use 2D spatial dimensions.
pub type Dimensions = [Scalar; 2];

/// General use 2D spatial point.
pub type Point = [Scalar; 2];

/// A cached widget's position for rendering.
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

    /// Set the width for the widget.
    fn width(self, width: Scalar) -> Self;

    /// Set the height for the widget.
    fn height(self, height: Scalar) -> Self;

    /// Get the width of the widget.
    fn get_width<C: CharacterCache>(&self, theme: &Theme, glyph_cache: &GlyphCache<C>) -> Scalar;

    /// Get the height of the widget.
    fn get_height(&self, theme: &Theme) -> Scalar;

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

    /// The dimensions for the widget.
    fn get_dimensions<C: CharacterCache>(&self, theme: &Theme, glyph_cache: &GlyphCache<C>)
        -> Dimensions
    {
        [self.get_width(theme, glyph_cache), self.get_height(theme)]
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


/// Some start and end position along a single axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Range {
    /// The start of some `Range` along an axis.
    pub start: Scalar,
    /// The end of some `Range` along an axis.
    pub end: Scalar,
}


impl Range {

    /// Construct a new `Range` from a given range, i.e. `Range::new(start..end)`.
    pub fn new(range: ::std::ops::Range<Scalar>) -> Range {
        Range { start: range.start, end: range.end }
    }

    /// Construct a new `Range` from a given length and its centered position.
    pub fn from_pos_and_len(pos: Scalar, len: Scalar) -> Range {
        let half_len = len / 2.0;
        let start = pos - half_len;
        let end = pos + half_len;
        Range::new(start..end)
    }

    /// The `start` value subtracted from the `end` value.
    pub fn magnitude(&self) -> Scalar {
        self.end - self.start
    }

    /// The absolute length of the Range aka the absolute magnitude.
    pub fn len(&self) -> Scalar {
        self.magnitude().abs()
    }

    /// Return the value directly between the start and end values.
    pub fn middle(&self) -> Scalar {
        (self.end + self.start) / 2.0
    }

    /// The current range with its start and end values swapped.
    pub fn invert(self) -> Range {
        Range { start: self.end, end: self.start }
    }

    /// Map the given Scalar from `Self` to some other given `Range`.
    pub fn map_value_to(&self, value: Scalar, other: &Range) -> Scalar {
        ::utils::map_range(value, self.start, self.end, other.start, other.end)
    }

    /// Shift the `Range` start and end points by a given `Scalar`.
    pub fn shift(self, amount: Scalar) -> Range {
        Range { start: self.start + amount, end: self.end + amount }
    }

    /// The direction of the Range represented as a normalised scalar.
    pub fn direction(&self) -> Scalar {
        if      self.start < self.end { 1.0 }
        else if self.start > self.end { -1.0 }
        else                          { 0.0 }
    }

    /// Converts the Range to an undirected Range. By ensuring that `start` <= `end`.
    /// If `start` > `end`, then the start and end points will be swapped.
    pub fn undirected(self) -> Range {
        if self.start > self.end { self.invert() } else { self }
    }

    /// The Range that encompasses both self and the given Range.
    /// The returned Range's `start` will always be <= its `end`.
    pub fn max(self, other: Range) -> Range {
        let start = self.start.min(self.end).min(other.start).min(other.end);
        let end = self.start.max(self.end).max(other.start).max(other.end);
        Range::new(start..end)
    }

    /// The Range that encompasses both self and the given Range.
    /// The returned Range will retain `self`'s original direction.
    pub fn max_directed(self, other: Range) -> Range {
        if self.start <= self.end { self.max(other) }
        else                      { self.max(other).invert() }
    }

    /// Is the given scalar within our range.
    pub fn is_over(&self, pos: Scalar) -> bool {
        let Range { start, end } = self.undirected();
        pos >= start && pos <= end
    }

    /// Shorten the Range from both ends by the given Scalar amount.
    pub fn sub_frame(self, frame: Scalar) -> Range {
        if self.direction() >= 0.0 {
            Range::new(self.start + frame .. self.end - frame)
        } else {
            Range::new(self.start - frame .. self.end + frame)
        }
    }

    /// Lengthen the Range from both ends by the given Scalar amount.
    pub fn add_frame(self, frame: Scalar) -> Range {
        if self.start <= self.end {
            Range::new(self.start - frame .. self.end + frame)
        } else {
            Range::new(self.start + frame .. self.end - frame)
        }
    }

    /// The Range with some padding given for each end.
    pub fn padded(self, pad_start: Scalar, pad_end: Scalar) -> Range {
        if self.start <= self.end {
            Range::new(self.start + pad_start .. self.end - pad_end)
        } else {
            Range::new(self.start - pad_start .. self.end + pad_end)
        }
    }

}

impl ::std::ops::Add<Range> for Range {
    type Output = Range;
    fn add(self, rhs: Range) -> Range {
        Range::new(self.start + rhs.start .. self.end + rhs.end)
    }
}

impl ::std::ops::Sub<Range> for Range {
    type Output = Range;
    fn sub(self, rhs: Range) -> Range {
        Range::new(self.start - rhs.start .. self.end - rhs.end)
    }
}



/// Start and end bounds on a single axis.
pub type Bounds = Range;

/// Defines a Rectangle's bounds across the x and y axes.
/// This is a conrod-specific Rectangle in that it's designed to help with layout.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rect {
    /// The start and end positions of the Rectangle on the x axis.
    pub x: Bounds,
    /// The start and end positions of the Rectangle on the y axis.
    pub y: Bounds,
}


impl Rect {

    /// Construct a Rect from a given `Point` and `Dimensions`.
    pub fn from_xy_dim(xy: Point, dim: Dimensions) -> Rect {
        Rect {
            x: Range::from_pos_and_len(xy[0], dim[0]),
            y: Range::from_pos_and_len(xy[1], dim[1]),
        }
    }

    /// The Rect that encompass the two given sets of Rect.
    pub fn max(self, other: Rect) -> Rect {
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

    /// Shift the Rect along the x axis.
    pub fn shift_x(self, x: Scalar) -> Rect {
        Rect { x: self.x.shift(x), ..self }
    }

    /// Shift the Rect along the y axis.
    pub fn shift_y(self, y: Scalar) -> Rect {
        Rect { y: self.y.shift(y), ..self }
    }

    /// Shift the Rect by the given Point.
    pub fn shift(self, xy: Point) -> Rect {
        self.shift_x(xy[0]).shift_y(xy[1])
    }

    /// Does the given point touch the Rectangle.
    pub fn is_over(&self, xy: Point) -> bool {
        self.x.is_over(xy[0]) && self.y.is_over(xy[1])
    }

    /// Shorten the x and y lengths by the given Scalar amount.
    pub fn sub_frame(self, frame: Scalar) -> Rect {
        Rect {
            x: self.x.sub_frame(frame),
            y: self.y.sub_frame(frame),
        }
    }

    /// Lengthen the x and y lengths by the given Scalar amount.
    pub fn add_frame(self, frame: Scalar) -> Rect {
        Rect {
            x: self.x.add_frame(frame),
            y: self.y.add_frame(frame),
        }
    }

    /// The Rect with some padding applied.
    pub fn padded(self, padding: Padding) -> Rect {
        Rect {
            x: self.x.padded(padding.left, padding.right),
            y: self.y.padded(padding.bottom, padding.top),
        }
    }

}

impl ::std::ops::Add<Rect> for Rect {
    type Output = Rect;
    fn add(self, rhs: Rect) -> Rect {
        Rect {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ::std::ops::Sub<Rect> for Rect {
    type Output = Rect;
    fn sub(self, rhs: Rect) -> Rect {
        Rect {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}



/// A function to simplify determining whether or not a point `xy` is over a rectangle.
/// `rect_xy` is the centered coordinatees of the rectangle.
pub fn is_over_rect(rect_xy: Point, rect_dim: Dimensions, xy: Point) -> bool {
    Rect::from_xy_dim(rect_xy, rect_dim).is_over(xy)
}

