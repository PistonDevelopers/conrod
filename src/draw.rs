
use graphics::{ BackEnd, ImageSize };

/// A trait to be implemented for all
/// drawable widget contexts.
pub trait Drawable<I: ImageSize> {
    fn draw<B: BackEnd<I>>(&mut self, graphics: &mut B);
}

