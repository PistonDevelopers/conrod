
use canvas::CanvasId;
use graphics::character::CharacterCache;
use graphics::math::Scalar;
use theme::Theme;
use ui::{GlyphCache, UiId};

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
    /// A position relative to some other Ui element.
    Relative(Scalar, Scalar, Option<UiId>),
    /// A direction relative to some other Ui element.
    Direction(Direction, Scalar, Option<UiId>),
    /// A position at a place on the current Canvas.
    Place(Place, Option<CanvasId>),
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
pub struct HorizontalAlign(pub Horizontal, pub Option<UiId>);

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
pub struct VerticalAlign(pub Vertical, pub Option<UiId>);

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

/// Place the widget at a position on the Canvas.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable, PartialEq, Eq)]
pub enum Place {
    /// Centre of the Canvas.
    Middle,
    /// Top left of the Canvas - pad_top + pad_left.
    TopLeft,
    /// Top right of the Canvas - pad_top - pad_right.
    TopRight,
    /// Bottom left of the Canvas + pad_bottom + pad_left.
    BottomLeft,
    /// Bottom right of the Canvas + pad_bottom - pad_right.
    BottomRight,
    /// Top centre of the Canvas - pad_top.
    MidTop,
    /// Bottom centre of the Canvas + pad_bottom.
    MidBottom,
    /// Left centre of the Canvas + pad_left.
    MidLeft,
    /// Right centre of the Canvas - pad_right.
    MidRight,
}

/// Widgets that are positionable.
pub trait Positionable: Sized {

    /// Set the Position.
    fn position(self, pos: Position) -> Self;

    /// Get the Position.
    fn get_position(&self) -> Position;

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

    /// Set the position relative to the widget with the given UiId.
    fn relative_to(self, ui_id: UiId, point: Point) -> Self {
        self.position(Position::Relative(point[0], point[1], Some(ui_id)))
    }

    /// Set the position relative to the widget with the given UiId.
    fn relative_xy_to(self, ui_id: UiId, x: Scalar, y: Scalar) -> Self {
        self.position(Position::Relative(x, y, Some(ui_id)))
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

    /// Set the position as below the widget with the given UiId.
    fn down_from(self, ui_id: UiId, pixels: Scalar) -> Self {
        self.position(Position::Direction(Direction::Down, pixels, Some(ui_id)))
    }

    /// Set the position as above the widget with the given UiId.
    fn up_from(self, ui_id: UiId, pixels: Scalar) -> Self {
        self.position(Position::Direction(Direction::Up, pixels, Some(ui_id)))
    }

    /// Set the position to the left of the widget with the given UiId.
    fn left_from(self, ui_id: UiId, pixels: Scalar) -> Self {
        self.position(Position::Direction(Direction::Left, pixels, Some(ui_id)))
    }

    /// Set the position to the right of the widget with the given UiId.
    fn right_from(self, ui_id: UiId, pixels: Scalar) -> Self {
        self.position(Position::Direction(Direction::Right, pixels, Some(ui_id)))
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
    fn align_left_of(self, other: UiId) -> Self {
        self.horizontal_align(HorizontalAlign(Horizontal::Left, Some(other)))
    }

    /// Align the position to the middle (only effective for Up or Down `Direction`s).
    fn align_middle_x_of(self, other: UiId) -> Self {
        self.horizontal_align(HorizontalAlign(Horizontal::Middle, Some(other)))
    }

    /// Align the position to the right (only effective for Up or Down `Direction`s).
    fn align_right_of(self, other: UiId) -> Self {
        self.horizontal_align(HorizontalAlign(Horizontal::Right, Some(other)))
    }

    /// Align the position to the top (only effective for Left or Right `Direction`s).
    fn align_top_of(self, other: UiId) -> Self {
        self.vertical_align(VerticalAlign(Vertical::Top, Some(other)))
    }

    /// Align the position to the middle (only effective for Left or Right `Direction`s).
    fn align_middle_y_of(self, other: UiId) -> Self {
        self.vertical_align(VerticalAlign(Vertical::Middle, Some(other)))
    }

    /// Align the position to the bottom (only effective for Left or Right `Direction`s).
    fn align_bottom_of(self, other: UiId) -> Self {
        self.vertical_align(VerticalAlign(Vertical::Bottom, Some(other)))
    }


    ///// `Place` methods. /////

    /// Place the widget at some position on the Canvas.
    fn place(self, place: Place, maybe_id: Option<CanvasId>) -> Self {
        self.position(Position::Place(place, maybe_id))
    }

    /// Place the widget in the middle of the given Canvas.
    fn middle_of(self, id: CanvasId) -> Self { self.place(Place::Middle, Some(id)) }

    /// Place the widget in the top left corner of the given Canvas.
    fn top_left_of(self, id: CanvasId) -> Self { self.place(Place::TopLeft, Some(id)) }

    /// Place the widget in the top right corner of the given Canvas.
    fn top_right_of(self, id: CanvasId) -> Self { self.place(Place::TopRight, Some(id)) }

    /// Place the widget in the bottom left corner of the given Canvas.
    fn bottom_left_of(self, id: CanvasId) -> Self { self.place(Place::BottomLeft, Some(id)) }

    /// Place the widget in the bottom right corner of the given Canvas.
    fn bottom_right_of(self, id: CanvasId) -> Self { self.place(Place::BottomRight, Some(id)) }

    /// Place the widget in the middle of the top edge of the given Canvas.
    fn mid_top_of(self, id: CanvasId) -> Self { self.place(Place::MidTop, Some(id)) }

    /// Place the widget in the middle of the bottom edge of the given Canvas.
    fn mid_bottom_of(self, id: CanvasId) -> Self { self.place(Place::MidBottom, Some(id)) }

    /// Place the widget in the middle of the left edge of the given Canvas.
    fn mid_left_of(self, id: CanvasId) -> Self { self.place(Place::MidLeft, Some(id)) }

    /// Place the widget in the middle of the right edge of the given Canvas.
    fn mid_right_of(self, id: CanvasId) -> Self { self.place(Place::MidRight, Some(id)) }

    /// Place the widget in the middle of the current Canvas.
    fn middle(self) -> Self { self.place(Place::Middle, None) }

    /// Place the widget in the top left corner of the current Canvas.
    fn top_left(self) -> Self { self.place(Place::TopLeft, None) }

    /// Place the widget in the top right corner of the current Canvas.
    fn top_right(self) -> Self { self.place(Place::TopRight, None) }

    /// Place the widget in the bottom left corner of the current Canvas.
    fn bottom_left(self) -> Self { self.place(Place::BottomLeft, None) }

    /// Place the widget in the bottom right corner of the current Canvas.
    fn bottom_right(self) -> Self { self.place(Place::BottomRight, None) }

    /// Place the widget in the middle of the top edge of the current Canvas.
    fn mid_top(self) -> Self { self.place(Place::MidTop, None) }

    /// Place the widget in the middle of the bottom edge of the current Canvas.
    fn mid_bottom(self) -> Self { self.place(Place::MidBottom, None) }

    /// Place the widget in the middle of the left edge of the current Canvas.
    fn mid_left(self) -> Self { self.place(Place::MidLeft, None) }

    /// Place the widget in the middle of the right edge of the current Canvas.
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

    /// Return the dimensions for the widget.
    fn get_dimensions<C: CharacterCache>(&self,
                                         theme: &Theme,
                                         glyph_cache: &GlyphCache<C>) -> Dimensions {
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
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Padding {
    /// Padding between the top of a Widget and the top of a Canvas.
    pub top: f64,
    /// Padding between the bottom of a Widget and the bottom of a Canvas.
    pub bottom: f64,
    /// Margin between the left of a Widget and the left of a Canvas.
    pub left: f64,
    /// Margin between the right of a Widget and the right of a Canvas.
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
    /// Margin between the y max Canvas and the outer edge of its frame.
    pub top: f64,
    /// Margin between the y min Canvas and the outer edge of its frame.
    pub bottom: f64,
    /// Margin between the x min Canvas and the outer edge of its frame.
    pub left: f64,
    /// Margin between the x max Canvas and the outer edge of its frame.
    pub right: f64,
}

