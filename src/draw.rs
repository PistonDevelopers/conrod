
use opengl_graphics::Gl;

/// A trait to be implemented for all
/// drawable widget contexts.
pub trait Drawable {
    fn draw(&mut self, gl: &mut Gl);
}

