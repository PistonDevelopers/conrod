use piston::quack::{ GetFrom, Get, Pair, Set, SetAt };
use dimensions::Dimensions;

/// A trait that indicates whether or not a widget
/// builder is positionable.
pub trait Shapeable {
    fn dimensions(self, width: f64, height: f64) -> Self;
    fn dim(self, dim: Dimensions) -> Self;
    fn width(self, width: f64) -> Self;
    fn height(self, height: f64) -> Self;
}

/// Size property.
#[derive(Copy)]
pub struct Size(pub Dimensions);

impl<T> Shapeable for T
    where
        (Size, T): Pair<Data = Size, Object = T> + SetAt + GetFrom
{
    #[inline]
    fn dimensions(self, width: f64, height: f64) -> Self {
        self.set(Size([width, height]))
    }
    #[inline]
    fn dim(self, dim: ::dimensions::Dimensions) -> Self {
        self.set(Size(dim))
    }
    #[inline]
    fn width(self, width: f64) -> Self {
        let Size([_, height]) = self.get();
        self.set(Size([width, height]))
    }
    #[inline]
    fn height(self, height: f64) -> Self {
        let Size([width, _]) = self.get();
        self.set(Size([width, height]))
    }
}
