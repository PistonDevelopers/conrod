
use envelope_editor::EnvelopePoint;

/// General use 2D spatial point.
pub type Point = [f64, ..2];

impl EnvelopePoint<f64, f64> for Point {
    /// Return the X value.
    fn get_x(&self) -> f64 { self[0] }
    /// Return the Y value.
    fn get_y(&self) -> f64 { self[1] }
    /// Return the X value.
    fn set_x(&mut self, x: f64) { self[0] = x }
    /// Return the Y value.
    fn set_y(&mut self, y: f64) { self[1] = y }
    /// Create a new Envelope Point.
    fn new(x: f64, y: f64) -> Point { [x, y] }
}

