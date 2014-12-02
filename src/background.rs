
use color::Color;
use draw::Drawable;
use graphics;
use opengl_graphics::Gl;
use ui_context::UiContext;

/// The context from which we'll draw the background.
pub struct BackgroundContext<'a> {
    uic: &'a mut UiContext,
    maybe_color: Option<Color>,
}

/// A trait to be implemented for the UiContext.
pub trait BackgroundBuilder {
    fn background<'a>(&'a mut self) -> BackgroundContext<'a>;
}

impl BackgroundBuilder for UiContext {
    fn background<'a>(&'a mut self) -> BackgroundContext<'a> {
        BackgroundContext {
            uic: self,
            maybe_color: None,
        }
    }
}

impl_colorable!(BackgroundContext)

impl<'a> Drawable for BackgroundContext<'a> {
    fn draw(&mut self, graphics: &mut Gl) {
        let (r, g, b, a) = self.maybe_color.unwrap_or(self.uic.theme.background_color).as_tuple();
        graphics::clear([r, g, b, a], graphics);
    }
}

