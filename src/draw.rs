
use opengl_graphics::Gl;
use UiContext;

/// A trait to be implemented for all
/// drawable widget contexts.
pub trait Drawable {
    fn draw(&mut self, uic: &mut UiContext, graphics: &mut Gl);
}
