
use envelope_editor::EnvelopePoint;
use std::default::Default;
use std::num::from_f32;

/// General use 3D spatial point. Although only
/// 2D widgets exist at the moment, this allows
/// for the possibility of 3D widgets in the
/// future.
#[deriving(Show, Clone, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Num + Copy> Point<T> {

    /// Constructor for a point.
    pub fn new(x: T, y: T, z: T) -> Point<T> {
        Point{ x: x, y: y, z: z }
    }

    /// Return the point as a vector.
    pub fn as_vec(&self) -> Vec<T> {
        vec![self.x, self.y, self.z]
    }

    /// Return the point as a tuple.
    pub fn as_tuple(&self) -> (T, T, T) {
        (self.x, self.y, self.z)
    }

}

impl<T: Num + Copy + ToPrimitive + FromPrimitive + PartialOrd + ToString>
EnvelopePoint<T, T> for Point<T> {
    /// Return the X value.
    fn get_x(&self) -> T { self.x }
    /// Return the Y value.
    fn get_y(&self) -> T { self.y }
    /// Return the X value.
    fn set_x(&mut self, x: T) { self.x = x }
    /// Return the Y value.
    fn set_y(&mut self, y: T) { self.y = y }
    /// Return the bezier curve depth (-1. to 1.) for the next interpolation.
    fn get_curve(&self) -> f32 { self.z.to_f32().unwrap() }
    /// Set the bezier curve depth (-1. to 1.) for the next interpolation.
    fn set_curve(&mut self, curve: f32) { self.z = from_f32(curve).unwrap(); }
    /// Create a new Envelope Point.
    fn new(x: T, y: T) -> Point<T> {
        Point::new(x, y, from_f32(0.0).unwrap())
    }
}

impl<T: Num + Copy + Default> Default for Point<T> {
    fn default() -> Point<T> {
        Point::new(Default::default(), Default::default(), Default::default())
    }
}

impl<T: Num + Copy> Add<Point<T>, Point<T>> for Point<T> {
    fn add(&self, rhs: &Point<T>) -> Point<T> {
        Point::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<T: Num + Copy> Sub<Point<T>, Point<T>> for Point<T> {
    fn sub(&self, rhs: &Point<T>) -> Point<T> {
        Point::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<T: Num + Copy> Mul<Point<T>, Point<T>> for Point<T> {
    fn mul(&self, rhs: &Point<T>) -> Point<T> {
        Point::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl<T: Num + Copy> Div<Point<T>, Point<T>> for Point<T> {
    fn div(&self, rhs: &Point<T>) -> Point<T> {
        Point::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

