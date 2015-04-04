
use graphics::Graphics;
use graphics::character::CharacterCache;
use Ui;

/// A trait to be implemented for all
/// drawable widget contexts.
pub trait Drawable {
    fn draw<B, C>(&mut self, ui: &mut Ui<C>, graphics: &mut B)
        where
            B: Graphics<Texture = <C as CharacterCache>::Texture>,
            C: CharacterCache
    ;
}
