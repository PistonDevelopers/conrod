
/// Widgets that implement a callback.
pub trait Callable<T> {

    /// Set a callback for Self. Note that the callback will not be evaluated until the
    /// `.set(&mut ui)` method is called, Be sure to call `set` soon after giving a callback
    /// as to not capture ownership of the upvars for too long (which will likely create
    /// annoying ownership issues).
    fn callback(self, cb: T) -> Self;

}
