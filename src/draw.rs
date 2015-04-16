
use graphics::Graphics;
use graphics::character::CharacterCache;
use Ui;

/// Widgets that are renderable.
pub trait Drawable {

    /// Draw a widget using the given graphics backend.
    fn draw<B, C>(&mut self, ui: &mut Ui<C>, graphics: &mut B)
        where
            B: Graphics<Texture = <C as CharacterCache>::Texture>,
            C: CharacterCache;

}
