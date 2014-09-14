
/// A trait that indicates whether or not a widget
/// builder is positionable.
pub trait Shapeable {
    fn dimensions(self, width: f64, height: f64) -> Self;
}

