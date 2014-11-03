
pub type ConrodResult<T> = Result<T, Error>;

/// An enum to represent various possible run-time errors that may occur.
#[deriving(Show, PartialEq, Eq)]
pub enum Error {
    FreetypeError(::freetype::error::Error),
}

