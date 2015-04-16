
use dimensions::Dimensions;

/// A trait that indicates whether or not a widget
/// builder is positionable.
pub trait Shapeable: Sized {
    fn get_dim(&self) -> Dimensions;
    fn dim(self, dim: Dimensions) -> Self;
    fn dimensions(self, width: f64, height: f64) -> Self {
        self.dim([width, height])
    }
    fn width(self, width: f64) -> Self {
        let size = self.get_dim();
        self.dim([width, size[1]])
    }
    fn height(self, height: f64) -> Self {
        let size = self.get_dim();
        self.dim([size[0], height])
    }
}
