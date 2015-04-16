
use dimensions::Dimensions;

/// Widgets that have some shape and dimension.
pub trait Shapeable: Sized {

    /// Return the dimensions of the widget.
    fn get_dim(&self) -> Dimensions;

    /// Set the dimensions for the widget.
    fn dim(self, dim: Dimensions) -> Self;

    /// Set the width and height for the widget.
    fn dimensions(self, width: f64, height: f64) -> Self {
        self.dim([width, height])
    }

    /// Set the width for the widget.
    fn width(self, width: f64) -> Self {
        let size = self.get_dim();
        self.dim([width, size[1]])
    }

    /// Set the height for the widget.
    fn height(self, height: f64) -> Self {
        let size = self.get_dim();
        self.dim([size[0], height])
    }

}
