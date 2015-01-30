
use graphics::BackEnd;
use graphics::character::CharacterCache;
use UiContext;

/// A trait to be implemented for all
/// drawable widget contexts.
pub trait Drawable {
    fn draw<B, C>(&mut self, uic: &mut UiContext<C>, graphics: &mut B)
        where
            B: BackEnd<Texture = <C as CharacterCache>::Texture>,
            C: CharacterCache
    ;
}
