
use Ui;
use widget;
use json::JsonValue;

pub use self::range::{Edge, Range};
pub use self::rect::{Corner, Rect};
//pub use self::matrix::Matrix;


//pub mod matrix;
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

/// The margin for some `Place`ment on either end of an axis.
pub type Margin = Scalar;

/// Represents either **Axis** in the 2-dimensional plane.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Axis {
    /// The horizontal plane's Axis.
    X,
    /// The vertical plane's Axis.
    Y,
}

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

impl Position {

    /// Converts this **Position** into a **JsonValue** representing it.
    /// All **Option<widget::Index>** values are set to null because it doesn't make sense to store
    /// them between sessions, since this is meant for using with themes.
    pub fn into_json(self) -> JsonValue {
        let mut value = JsonValue::new_object();
        match self {
            Position::Absolute(pos) => value["absolute"] = pos.into(),
            Position::Relative(pos, _index) => value["relative"] = object!{
                "offset" => pos,
                "parent" => ::json::Null
            },
            Position::Align(align, _index) => value["align"] = object!{
                "alignment" => align.into_json(),
                "parent" => ::json::Null
            },
            Position::Direction(direction, pos, _index) => value["direction"] = object!{
                "direction" => direction.into_json(),
                "offset" => pos,
                "parent" => ::json::Null
            },
            Position::Place(place, _index ) => value["place"] = object!{
                "place" => place.into_json(),
                "index" => ::json::Null
            }
        }
        value
    }
}

/// Directionally positioned, normally relative to some other widget.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Positioned forwards (*positive* **Scalar**) along some **Axis**.
    Forwards,
    /// Positioned backwards (*negative* **Scalar**) along some **Axis**.
    Backwards,
}

impl Direction {
    /// Converts this **Direction** into a string representing it for use in theme serialization.
    pub fn into_json(self) -> &'static str {
        match self {
            Direction::Forwards => "Forwards",
            Direction::Backwards => "Backwards"
        }
    }
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

impl Align {
    /// Converts this **Align** into a string representing it, for use in JSON
    pub fn into_json(self) -> &'static str {
        match self {
            Align::Start => "Start",
            Align::Middle => "Middle",
            Align::End => "End"
        }
    }
}

/// Place the widget at a position on some other widget.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Place {
    /// Place upon the **Start** of the Widget's `kid_area`.
    Start(Option<Margin>),
    /// Place upon the **Middle** of the Widget's `kid_area`.
    Middle,
    /// Place upon the **End** of the Widget's `kid_area`.
    End(Option<Margin>),
}

impl Place {
    /// Converts this **Place** into a **JsonValue** for use in theme serialization.
    pub fn into_json(self) -> JsonValue {
        let mut data = JsonValue::new_object();
        match self {
            Place::Start(margin) => data["start"] = margin.into(),
            Place::Middle => data["middle"] = ::json::Null,
            Place::End(margin) => data["end"] = margin.into()
        }
        data
    }
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
    ///
    /// The `Option<Scalar>` is an optional padding argument which when `Some`, will subtract the
    /// scalar from both ends of the other widget's dimension.
    Of(widget::Index, Option<Scalar>),
    /// The dimension should match that of the `kid_area` of the widget at the given index.
    ///
    /// The `Option<Scalar>` is an optional padding argument which when `Some`, will subtract the
    /// scalar from both ends of the other widget's dimension.
    KidAreaOf(widget::Index, Option<Scalar>),
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
    fn get_x_position(&self, ui: &Ui) -> Position;

    /// Get the **Position** along the *y* axis.
    fn get_y_position(&self, ui: &Ui) -> Position;

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
        self.x_position(Position::Direction(direction, x, None))
    }

    /// Build with the **Position** along the *y* axis as some distance from another widget.
    fn y_direction(self, direction: Direction, y: Scalar) -> Self {
        self.y_position(Position::Direction(direction, y, None))
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
        self.x_position(Position::Direction(direction, x, Some(other.into())))
    }

    /// Build with the **Position** along the *y* axis as some distance from the given widget.
    fn y_direction_from<I>(self, other: I, direction: Direction, y: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.y_position(Position::Direction(direction, y, Some(other.into())))
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
        self.x_place_on(other, Place::Start(None)).y_place_on(other, Place::End(None))
    }

    /// Place the widget in the top left corner of the given Widget with the given margin between
    /// both edges.
    fn top_left_with_margin_on<I>(self, other: I, mgn: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.x_place_on(other, Place::Start(Some(mgn))).y_place_on(other, Place::End(Some(mgn)))
    }

    /// Place the widget in the top left corner of the given Widget with the given margins between
    /// each respective edge.
    fn top_left_with_margins_on<I>(self, other: I, top: Scalar, left: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.x_place_on(other, Place::Start(Some(left))).y_place_on(other, Place::End(Some(top)))
    }

    /// Place the widget in the top right corner of the given Widget.
    fn top_right_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::End(None)).y_place_on(other, Place::End(None))
    }

    /// Place the widget in the top right corner of the given Widget with the given margin
    /// between both edges.
    fn top_right_with_margin_on<I>(self, other: I, mgn: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.x_place_on(other, Place::End(Some(mgn))).y_place_on(other, Place::End(Some(mgn)))
    }

    /// Place the widget in the top right corner of the given Widget with the given margins between
    /// each respective edge.
    fn top_right_with_margins_on<I>(self, other: I, top: Scalar, right: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.x_place_on(other, Place::End(Some(right))).y_place_on(other, Place::End(Some(top)))
    }

    /// Place the widget in the bottom left corner of the given Widget.
    fn bottom_left_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::Start(None)).y_place_on(other, Place::Start(None))
    }

    /// Place the widget in the bottom left corner of the given Widget with the given margin
    /// between both edges.
    fn bottom_left_with_margin_on<I>(self, other: I, mgn: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.x_place_on(other, Place::Start(Some(mgn))).y_place_on(other, Place::Start(Some(mgn)))
    }

    /// Place the widget in the bottom left corner of the given Widget with the given margins
    /// between each respective edge.
    fn bottom_left_with_margins_on<I>(self, other: I, bottom: Scalar, left: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.x_place_on(other, Place::Start(Some(left))).y_place_on(other, Place::Start(Some(bottom)))
    }

    /// Place the widget in the bottom right corner of the given Widget.
    fn bottom_right_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::End(None)).y_place_on(other, Place::Start(None))
    }

    /// Place the widget in the bottom right corner of the given Widget with the given margin
    /// between both edges.
    fn bottom_right_with_margin_on<I>(self, other: I, mgn: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.x_place_on(other, Place::End(Some(mgn))).y_place_on(other, Place::Start(Some(mgn)))
    }

    /// Place the widget in the bottom right corner of the given Widget with the given margins
    /// between each respective edge.
    fn bottom_right_with_margins_on<I>(self, other: I, bottom: Scalar, right: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.x_place_on(other, Place::End(Some(right))).y_place_on(other, Place::Start(Some(bottom)))
    }

    /// Place the widget in the middle of the top edge of the given Widget.
    fn mid_top_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::Middle).y_place_on(other, Place::End(None))
    }

    /// Place the widget in the middle of the top edge of the given Widget with the given margin
    /// between the edges.
    fn mid_top_with_margin_on<I>(self, other: I, mgn: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.x_place_on(other, Place::Middle).y_place_on(other, Place::End(Some(mgn)))
    }

    /// Place the widget in the middle of the bottom edge of the given Widget.
    fn mid_bottom_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::Middle).y_place_on(other, Place::Start(None))
    }

    /// Place the widget in the middle of the bottom edge of the given Widget with the given margin
    /// between the edges.
    fn mid_bottom_with_margin_on<I>(self, other: I, mgn: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.x_place_on(other, Place::Middle).y_place_on(other, Place::Start(Some(mgn)))
    }

    /// Place the widget in the middle of the left edge of the given Widget.
    fn mid_left_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::Start(None)).y_place_on(other, Place::Middle)
    }

    /// Place the widget in the middle of the left edge of the given Widget with the given margin
    /// between the edges.
    fn mid_left_with_margin_on<I>(self, other: I, mgn: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.x_place_on(other, Place::Start(Some(mgn))).y_place_on(other, Place::Middle)
    }

    /// Place the widget in the middle of the right edge of the given Widget.
    fn mid_right_of<I: Into<widget::Index> + Copy>(self, other: I) -> Self {
        self.x_place_on(other, Place::End(None)).y_place_on(other, Place::Middle)
    }

    /// Place the widget in the middle of the right edge of the given Widget with the given margin
    /// between the edges.
    fn mid_right_with_margin_on<I>(self, other: I, mgn: Scalar) -> Self
        where I: Into<widget::Index> + Copy,
    {
        self.x_place_on(other, Place::End(Some(mgn))).y_place_on(other, Place::Middle)
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
        self.x_place(Place::Start(None)).y_place(Place::End(None))
    }

    /// Place the widget in the top left corner of the current parent Widget with the given margin
    /// between both edges.
    fn top_left_with_margin(self, mgn: Scalar) -> Self {
        self.x_place(Place::Start(Some(mgn))).y_place(Place::End(Some(mgn)))
    }

    /// Place the widget in the top left corner of the current parent Widget with the given margins
    /// between each respective edge.
    fn top_left_with_margins(self, top: Scalar, left: Scalar) -> Self {
        self.x_place(Place::Start(Some(left))).y_place(Place::End(Some(top)))
    }

    /// Place the widget in the top right corner of the current parent Widget.
    fn top_right(self) -> Self {
        self.x_place(Place::End(None)).y_place(Place::End(None))
    }

    /// Place the widget in the top right corner of the current parent Widget with the given margin
    /// between both edges.
    fn top_right_with_margin(self, mgn: Scalar) -> Self {
        self.x_place(Place::End(Some(mgn))).y_place(Place::End(Some(mgn)))
    }

    /// Place the widget in the top right corner of the current parent Widget with the given margins
    /// between each respective edge.
    fn top_right_with_margins(self, top: Scalar, right: Scalar) -> Self {
        self.x_place(Place::End(Some(right))).y_place(Place::End(Some(top)))
    }

    /// Place the widget in the bottom left corner of the current parent Widget.
    fn bottom_left(self) -> Self {
        self.x_place(Place::Start(None)).y_place(Place::Start(None))
    }

    /// Place the widget in the bottom left corner of the current parent Widget with the given
    /// margin between both edges.
    fn bottom_left_with_margin(self, mgn: Scalar) -> Self {
        self.x_place(Place::Start(Some(mgn))).y_place(Place::Start(Some(mgn)))
    }

    /// Place the widget in the bottom left corner of the current parent Widget with the given
    /// margins between each respective edge.
    fn bottom_left_with_margins(self, bottom: Scalar, left: Scalar) -> Self {
        self.x_place(Place::Start(Some(left))).y_place(Place::Start(Some(bottom)))
    }

    /// Place the widget in the bottom right corner of the current parent Widget.
    fn bottom_right(self) -> Self {
        self.x_place(Place::End(None)).y_place(Place::Start(None))
    }

    /// Place the widget in the bottom right corner of the current parent Widget with the given
    /// margin between both edges.
    fn bottom_right_with_margin(self, mgn: Scalar) -> Self {
        self.x_place(Place::End(Some(mgn))).y_place(Place::Start(Some(mgn)))
    }

    /// Place the widget in the bottom right corner of the current parent Widget with the given
    /// margins between each respective edge.
    fn bottom_right_with_margins(self, bottom: Scalar, right: Scalar) -> Self {
        self.x_place(Place::End(Some(right))).y_place(Place::Start(Some(bottom)))
    }

    /// Place the widget in the middle of the top edge of the current parent Widget.
    fn mid_top(self) -> Self {
        self.x_place(Place::Middle).y_place(Place::End(None))
    }

    /// Place the widget in the middle of the top edge of the current parent Widget with the given
    /// margin from the edge.
    fn mid_top_with_margin(self, mgn: Scalar) -> Self {
        self.x_place(Place::Middle).y_place(Place::End(Some(mgn)))
    }

    /// Place the widget in the middle of the bottom edge of the current parent Widget.
    fn mid_bottom(self) -> Self {
        self.x_place(Place::Middle).y_place(Place::Start(None))
    }

    /// Place the widget in the middle of the bottom edge of the current parent Widget with the
    /// given margin from the edge.
    fn mid_bottom_with_margin(self, mgn: Scalar) -> Self {
        self.x_place(Place::Middle).y_place(Place::Start(Some(mgn)))
    }

    /// Place the widget in the middle of the left edge of the current parent Widget.
    fn mid_left(self) -> Self {
        self.x_place(Place::Start(None)).y_place(Place::Middle)
    }

    /// Place the widget in the middle of the left edge of the current parent Widget with the
    /// given margin from the edge.
    fn mid_left_with_margin(self, mgn: Scalar) -> Self {
        self.x_place(Place::Start(Some(mgn))).y_place(Place::Middle)
    }

    /// Place the widget in the middle of the right edge of the current parent Widget.
    fn mid_right(self) -> Self {
        self.x_place(Place::End(None)).y_place(Place::Middle)
    }

    /// Place the widget in the middle of the right edge of the current parent Widget with the
    /// given margin from the edge.
    fn mid_right_with_margin(self, mgn: Scalar) -> Self {
        self.x_place(Place::End(Some(mgn))).y_place(Place::Middle)
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
    fn get_x_dimension(&self, ui: &Ui) -> Dimension;

    /// The widget's length along the y axis as a Dimension.
    fn get_y_dimension(&self, ui: &Ui) -> Dimension;

    // Provided defaults.

    /// Set the absolute width for the widget.
    fn w(self, w: Scalar) -> Self {
        self.x_dimension(Dimension::Absolute(w))
    }

    /// Set the absolute height for the widget.
    fn h(self, h: Scalar) -> Self {
        self.y_dimension(Dimension::Absolute(h))
    }

    /// Set the dimensions for the widget.
    fn wh(self, wh: Dimensions) -> Self {
        self.w(wh[0]).h(wh[1])
    }

    /// Set the width and height for the widget.
    fn w_h(self, width: Scalar, height: Scalar) -> Self {
        self.wh([width, height])
    }

    /// Set the width as the width of the widget at the given index.
    fn w_of<I: Into<widget::Index>>(self, idx: I) -> Self {
        self.x_dimension(Dimension::Of(idx.into(), None))
    }

    /// Set the width as the width of the widget at the given index padded at both ends by the
    /// given Scalar.
    fn padded_w_of<I: Into<widget::Index>>(self, idx: I, pad: Scalar) -> Self {
        self.x_dimension(Dimension::Of(idx.into(), Some(pad)))
    }

    /// Set the height as the height of the widget at the given index.
    fn h_of<I: Into<widget::Index>>(self, idx: I) -> Self {
        self.y_dimension(Dimension::Of(idx.into(), None))
    }

    /// Set the height as the height of the widget at the given index padded at both ends by the
    /// given Scalar.
    fn padded_h_of<I: Into<widget::Index>>(self, idx: I, pad: Scalar) -> Self {
        self.y_dimension(Dimension::Of(idx.into(), Some(pad)))
    }

    /// Set the dimensions as the dimensions of the widget at the given index.
    fn wh_of<I: Into<widget::Index> + Copy>(self, idx: I) -> Self {
        self.w_of(idx).h_of(idx)
    }

    /// Set the dimensions as the dimensions of the widget at the given index with all four edges
    /// padded by the given scalar.
    fn padded_wh_of<I: Into<widget::Index> + Copy>(self, idx: I, pad: Scalar) -> Self {
        self.padded_w_of(idx, pad).padded_h_of(idx, pad)
    }

    /// Set the width as the width of the padded area of the widget at the given index.
    fn kid_area_w_of<I: Into<widget::Index>>(self, idx: I) -> Self {
        self.x_dimension(Dimension::KidAreaOf(idx.into(), None))
    }

    /// Set the width as the `KidArea` width for the widget at the given index, padded at both ends
    /// by the given scalar.
    fn padded_kid_area_w_of<I: Into<widget::Index>>(self, idx: I, pad: Scalar) -> Self {
        self.x_dimension(Dimension::KidAreaOf(idx.into(), Some(pad)))
    }

    /// Set the height as the `KidArea` height of the widget at the given index.
    fn kid_area_h_of<I: Into<widget::Index>>(self, idx: I) -> Self {
        self.y_dimension(Dimension::KidAreaOf(idx.into(), None))
    }

    /// Set the height as the `KidArea` height of the widget at the given index, padded at both
    /// ends by the given scalar.
    fn padded_kid_area_h_of<I: Into<widget::Index>>(self, idx: I, pad: Scalar) -> Self {
        self.y_dimension(Dimension::KidAreaOf(idx.into(), Some(pad)))
    }

    /// Set the dimensions as the `KidArea` dimensions of the widget at the given index.
    fn kid_area_wh_of<I: Into<widget::Index> + Copy>(self, idx: I) -> Self {
        self.kid_area_w_of(idx).kid_area_h_of(idx)
    }

    /// Set the dimensions as the `KidArea` dimensions of the widget at the given index, padded at
    /// all four edges by the given scalar.
    fn padded_kid_area_wh_of<I: Into<widget::Index> + Copy>(self, idx: I, pad: Scalar) -> Self {
        self.padded_kid_area_w_of(idx, pad).padded_kid_area_h_of(idx, pad)
    }

    /// Get the absolute width of the widget as a Scalar value.
    fn get_w(&self, ui: &Ui) -> Option<Scalar> {
        match self.get_x_dimension(ui) {
            Dimension::Absolute(width) => Some(width),
            Dimension::Of(idx, None) => ui.w_of(idx),
            Dimension::Of(idx, Some(pad)) => ui.w_of(idx).map(|w| w - pad * 2.0),
            Dimension::KidAreaOf(idx, None) => ui.kid_area_of(idx).map(|r| r.w()),
            Dimension::KidAreaOf(idx, Some(pad)) => ui.kid_area_of(idx).map(|r| r.w() - pad * 2.0),
        }
    }

    /// Get the height of the widget.
    fn get_h(&self, ui: &Ui) -> Option<Scalar> {
        match self.get_y_dimension(ui) {
            Dimension::Absolute(height) => Some(height),
            Dimension::Of(idx, None) => ui.h_of(idx),
            Dimension::Of(idx, Some(pad)) => ui.h_of(idx).map(|w| w - pad * 2.0),
            Dimension::KidAreaOf(idx, None) => ui.kid_area_of(idx).map(|r| r.h()),
            Dimension::KidAreaOf(idx, Some(pad)) => ui.kid_area_of(idx).map(|r| r.h() - pad * 2.0),
        }
    }

    /// The dimensions for the widget.
    fn get_wh(&self, ui: &Ui) -> Option<Dimensions> {
        self.get_w(ui).and_then(|w| self.get_h(ui).map(|h| [w, h]))
    }

}

/// The distance between the inner edge of a border and the outer edge of the inner content.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Padding {
    /// Padding on the start and end of the *x* axis.
    pub x: Range,
    /// Padding on the start and end of the *y* axis.
    pub y: Range,
}

impl Padding {
    /// No padding.
    pub fn none() -> Padding {
        Padding {
            x: Range::new(0.0, 0.0),
            y: Range::new(0.0, 0.0),
        }
    }
}
