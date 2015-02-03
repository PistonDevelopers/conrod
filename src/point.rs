
use envelope_editor::EnvelopePoint;
use graphics::vecmath::Scalar;

/// General use 2D spatial point.
pub type Point = [Scalar; 2];

impl EnvelopePoint for Point {
    type X = Scalar;
    type Y = Scalar;
    /// Return the X value.
    fn get_x(&self) -> Scalar { self[0] }
    /// Return the Y value.
    fn get_y(&self) -> Scalar { self[1] }
    /// Return the X value.
    fn set_x(&mut self, x: Scalar) { self[0] = x }
    /// Return the Y value.
    fn set_y(&mut self, y: Scalar) { self[1] = y }
    /// Create a new Envelope Point.
    fn new(x: Scalar, y: Scalar) -> Point { [x, y] }
}

