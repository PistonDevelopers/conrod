
use color::Color;
use draw::Drawable;
use graphics::{
    AddColor,
    Context,
    BackEnd,
    Draw,
    ImageSize,
};
use ui_context::UiContext;

/// The context from which we'll draw the background.
pub struct BackgroundContext<'a, T: 'a> {
    uic: &'a mut UiContext<T>,
    maybe_color: Option<Color>,
}

/// A trait to be implemented for the UiContext.
pub trait BackgroundBuilder<'a, T> {
    fn background(&'a mut self) -> BackgroundContext<'a, T>;
}

impl<'a, T> BackgroundBuilder<'a, T> for UiContext<T> {
    fn background(&'a mut self) -> BackgroundContext<'a, T> {
        BackgroundContext {
            uic: self,
            maybe_color: None,
        }
    }
}

impl_colorable!(BackgroundContext, T)

impl<'a, T: ImageSize> Drawable<T> for BackgroundContext<'a, T> {
    fn draw<B: BackEnd<T>>(&mut self, graphics: &mut B) {
        let (r, g, b, a) = self.maybe_color.unwrap_or(self.uic.theme.background_color).as_tuple();
        Context::abs(self.uic.win_w, self.uic.win_h)
            .rgba(r, g, b, a)
            .draw(graphics)
    }
}

