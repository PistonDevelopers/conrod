
use point::Point;
use ui::{UiId, Ui};

/// A cached widget's position for rendering.
#[derive(Copy, Clone, Debug)]
pub enum Position {
    /// A specific position.
    Absolute(Point),
    /// A position relative to some other widget.
    Relative(Point, Option<UiId>),
    /// A direction relative to some other widget.
    Direction(Direction, Option<UiId>),
}

impl Position {
    /// The default widget Position.
    pub fn default() -> Position{
        Position::Relative(Direction::Down, None)
    }
}

/// The layout direction for a relatively positioned widget.
pub enum Direction {
    /// Above.
    Up(f64),
    /// Below.
    Down(f64),
    /// To the left of.
    Left(f64),
    /// To the right of.
    Right(f64),
}

/// Widgets that are positionable.
pub trait Positionable: Sized {

    /// Set the Position.
    fn position(self, pos: Position) -> Self;

    /// Set the position with some Point.
    fn point(self, point: Point) -> Self {
        self.position(Position::Absolute(point))
    }

    /// Set the position with XY co-ords.
    fn xy(self, x: f64, y: f64) -> Self {
        self.position(Position::Absolute([x, y]))
    }

    /// Set the point relative to the previous widget.
    fn relative(self, point: Point) -> Self {
        self.position(Position::Relative(point, None))
    }

    /// Set the xy relative to the previous widget.
    fn relative_xy(self, x: f64, y: f64) -> Self {
        self.position(Position::Relative([x, y], None))
    }

    /// Set the position relative to the widget with the given UiId.
    fn relative_to(self, ui_id: UiId, point: Point) -> Self {
        self.position(Position::Relative(point, Some(ui_id)))
    }

    /// Set the position relative to the widget with the given UiId.
    fn relative_xy_to(self, ui_id: UiId, x: f64, y: f64) -> Self {
        self.position(Position::Relative([x, y], Some(ui_id)))
    }

    /// Set the position as below the previous widget.
    fn down(self, pixels: f64) -> Self {
        self.position(Position::Direction(Direction::Down(pixels), None))
    }

    /// Set the position as above the previous widget.
    fn up(self, pixels: f64) -> Self {
        self.position(Position::Direction(Direction::Up(pixels), None))
    }

    /// Set the position to the left of the previous widget.
    fn left(self, pixels: f64) -> Self {
        self.position(Position::Direction(Direction::Left(pixels), None))
    }

    /// Set the position to the right of the previous widget.
    fn right(self, pixels: f64) -> Self {
        self.position(Position::Direction(Direction::Right(pixels), None))
    }

    /// Set the position as below the widget with the given UiId.
    fn down_from(self, ui_id: UiId, pixels: f64) -> Self {
        self.position(Position::Direction(Direction::Down(pixels), Some(ui_id)))
    }

    /// Set the position as above the widget with the given UiId.
    fn up_from(self, ui_id: UiId, pixels: f64) -> Self {
        self.position(Position::Direction(Direction::Up(pixels), Some(ui_id)))
    }

    /// Set the position to the left of the widget with the given UiId.
    fn left_from(self, ui_id: UiId, pixels: f64) -> Self {
        self.position(Position::Direction(Direction::Left(pixels), Some(ui_id)))
    }

    /// Set the position to the right of the widget with the given UiId.
    fn right_from(self, ui_id: UiId, pixels: f64) -> Self {
        self.position(Position::Direction(Direction::Right(pixels), Some(ui_id)))
    }

}
