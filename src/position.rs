
use graphics::math::Scalar;
use ui::UiId;

/// The depth at which the widget will be rendered. This determines the order of rendering where
/// widgets with a greater depth will be rendered first. 0.0 is the default depth.
pub type Depth = f32;

/// General use 2D spatial dimensions.
pub type Dimensions = [Scalar; 2];

/// General use 2D spatial point.
pub type Point = [Scalar; 2];

/// A cached widget's position for rendering.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum Position {
    /// A specific position.
    Absolute(Scalar, Scalar),
    /// A position relative to some other widget.
    Relative(Scalar, Scalar, Option<UiId>),
    /// A direction relative to some other widget.
    Direction(Direction, Scalar, Option<UiId>),
}

impl Position {
    /// The default widget Position.
    pub fn default() -> Position{
        Position::Direction(Direction::Down, 20.0, None)
    }
}

/// Directionally positioned, relative to another widget.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
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

/// The horizontal alignment of a widget positioned relatively to another widget on the y axis.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable, PartialEq)]
pub enum HorizontalAlign {
    /// Align the left edges of the widgets.
    Left,
    /// Align the centres of the widgets' closest parallel edges.
    Middle,
    /// Align the right edges of the relative widgets.
    Right,
}

/// The vertical alignment of a widget positioned relatively to another widget on the x axis.
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable, PartialEq)]
pub enum VerticalAlign {
    /// Align the top edges of the widgets.
    Top,
    /// Align the centres of the widgets' closest parallel edges.
    Middle,
    /// Align the bottom edges of the widgets.
    Bottom,
}

/// Widgets that are positionable.
pub trait Positionable: Sized {

    /// Set the Position.
    fn position(self, pos: Position) -> Self;

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

    ///// Alignment methods. /////

    /// Align the position horizontally (only effective for Up or Down `Direction`s).
    fn horizontal_align(self, align: HorizontalAlign) -> Self;

    /// Align the position vertically (only effective for Left or Right `Direction`s).
    fn vertical_align(self, align: VerticalAlign) -> Self;

    /// Align the position to the left (only effective for Up or Down `Direction`s).
    fn align_left(self) -> Self {
        self.horizontal_align(HorizontalAlign::Left)
    }

    /// Align the position to the middle (only effective for Up or Down `Direction`s).
    fn align_middle_x(self) -> Self {
        self.horizontal_align(HorizontalAlign::Middle)
    }

    /// Align the position to the right (only effective for Up or Down `Direction`s).
    fn align_right(self) -> Self {
        self.horizontal_align(HorizontalAlign::Right)
    }

    /// Align the position to the top (only effective for Left or Right `Direction`s).
    fn align_top(self) -> Self {
        self.vertical_align(VerticalAlign::Top)
    }

    /// Align the position to the middle (only effective for Left or Right `Direction`s).
    fn align_middle_y(self) -> Self {
        self.vertical_align(VerticalAlign::Middle)
    }

    /// Align the position to the bottom (only effective for Left or Right `Direction`s).
    fn align_bottom(self) -> Self {
        self.vertical_align(VerticalAlign::Bottom)
    }

}

/// Widgets that support different dimensions.
pub trait Sizeable: Sized {

    /// Set the width for the widget.
    fn width(self, width: Scalar) -> Self;

    /// Set the height for the widget.
    #[inline]
    fn height(self, height: Scalar) -> Self;

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

