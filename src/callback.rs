use piston::quack::{ Pair, Set, SetAt };

/// A trait for widgets who implement a callback of
pub trait Callable<T> {
    fn callback(self, cb: T) -> Self;
}

/// Callback property.
pub struct Callback<T>(pub T);

impl<T, U> Callable<U> for T
    where
        (Callback<U>, T): Pair<Data = Callback<U>, Object = T> + SetAt
{
    fn callback(self, cb: U) -> Self {
        self.set(Callback(cb))
    }
}
