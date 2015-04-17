
use point::Point;
use ui::{UiId, Ui};

/// A cached widget's position for rendering.
#[derive(Copy, Clone, Debug)]
pub enum Position {
    /// A specific position.
    Absolute(Point),
    /// A position relative to some other widget.
    Relative(Direction, Option<UiId>),
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
    Up,
    /// Below.
    Down,
    /// To the left of.
    Left,
    /// To the right of.
    Right,
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

    /// Set the position as below the previous widget.
    fn down(self) -> Self {
        self.position(Position::Relative(Direction::Down, None))
    }

    /// Set the position as above the previous widget.
    fn up(self) -> Self {
        self.position(Position::Relative(Direction::Up, None))
    }

    /// Set the position to the left of the previous widget.
    fn left(self) -> Self {
        self.position(Position::Relative(Direction::Left, None))
    }

    /// Set the position to the right of the previous widget.
    fn right(self) -> Self {
        self.position(Position::Relative(Direction::Right, None))
    }

    /// Set the position as below the widget with the given UiId.
    fn down_from(self, ui_id: UiId) -> Self {
        self.position(Position::Relative(Direction::Down, Some(ui_id)))
    }

    /// Set the position as above the widget with the given UiId.
    fn up_from(self, ui_id: UiId) -> Self {
        self.position(Position::Relative(Direction::Up, Some(ui_id)))
    }

    /// Set the position to the left of the widget with the given UiId.
    fn left_from(self, ui_id: UiId) -> Self {
        self.position(Position::Relative(Direction::Left, Some(ui_id)))
    }

    /// Set the position to the right of the widget with the given UiId.
    fn right_from(self, ui_id: UiId) -> Self {
        self.position(Position::Relative(Direction::Right, Some(ui_id)))
    }

}
