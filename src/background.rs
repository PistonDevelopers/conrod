
use color::Color;
use graphics;
use opengl_graphics::Gl;
use ui_context::UiContext;
use internal;
use MaybeColor;

/// A background.
#[derive(Copy)]
pub struct Background {
    maybe_color: Option<internal::Color>,
}

impl Background {
    /// Creates a new background.
    pub fn new() -> Background {
        Background {
            maybe_color: None
        }
    }

    pub fn draw(&self, uic: &UiContext, graphics: &mut Gl) {
        let col = self.maybe_color
            .unwrap_or(uic.theme.background_color.0);
        graphics::clear(col, graphics);
    }
}

quack! {
    bg: Background[]
    get:
        fn () -> MaybeColor { MaybeColor(bg.maybe_color) }
    set:
        fn (val: Color) { bg.maybe_color = Some(val.0) }
    action:
}
