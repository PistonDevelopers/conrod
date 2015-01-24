use quack::{ GetFrom, SetAt };

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

impl SetAt for (Color, Background) {
    type Property = Color;
    type Object = Background;

    fn set_at(Color(color): Color, background: &mut Background) {
        background.maybe_color = Some(color);
    }
}

impl GetFrom for (MaybeColor, Background) {
    type Property = MaybeColor;
    type Object = Background;

    fn get_from(background: &Background) -> MaybeColor {
        MaybeColor(background.maybe_color)
    }
}
