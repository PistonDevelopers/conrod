
use internal::Dimensions;

/// A trait that indicates whether or not a widget
/// builder is positionable.
pub trait Shapeable {
    fn dimensions(self, width: f64, height: f64) -> Self;
    fn dim(self, dim: Dimensions) -> Self;
    fn width(self, width: f64) -> Self;
    fn height(self, height: f64) -> Self;
}
