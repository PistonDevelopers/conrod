
use color::Color;
use draw::Drawable;
use graphics;
use opengl_graphics::Gl;
use ui_context::UiContext;

/// The context from which we'll draw the background.
#[derive(Copy)]
pub struct Background {
    maybe_color: Option<Color>,
}

impl Background {
    pub fn new() -> Background {
        Background {
            maybe_color: None,
        }
    }
}

quack! {
    bg: Background[]
    get:
    set:
        fn (val: Color) { bg.maybe_color = Some(val) }
    action:
}

impl Drawable for Background {
    fn draw(&mut self, uic: &mut UiContext, graphics: &mut Gl) {
        let Color(col) = self.maybe_color
            .unwrap_or(uic.theme.background_color);
        graphics::clear(col, graphics);
    }
}
