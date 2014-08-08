
use std::default::Default;

/// General use 3D spatial point. Although only
/// 2D widgets exist at the moment, this allows
/// for the possibility of 3D widgets in the
/// future.
#[deriving(Show, Clone)]
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

