/// A trait for widgets who implement a callback of
pub trait Callable<T> {
    fn callback(self, cb: T) -> Self;
}
