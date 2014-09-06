
/// A trait that indicates whether or not a widget
/// builder is positionable.
pub trait Positionable {
    fn position(self, x: f64, y: f64) -> Self;
}

